use base64::{Engine as _, engine::general_purpose::STANDARD};
use reqwest::Client;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct Profile {
    id: String,
    name: String,
    properties: Vec<Property>,
}

#[derive(Deserialize)]
struct Property {
    name: String,
    value: String,
    signature: Option<String>,
}

#[derive(Deserialize)]
struct TexturePayload {
    textures: Textures,
}

#[derive(Deserialize)]
struct Textures {
    skin: Option<SkinInfo>,
}

#[derive(Deserialize)]
struct SkinInfo {
    url: String,
    metadata: Option<Metadata>,
}

#[derive(Deserialize)]
struct Metadata {
    model: Option<String>,
}

/// Retrieves the Base64-encoded texture value and its signature for a given Minecraft player UUID.
///
/// # Arguments
///
/// * `uuid` - The player's UUID from the `uuid` crate.
///
/// # Returns
///
/// A tuple containing the Base64 texture `value` and its `signature`.
pub async fn get_skin_and_signature(
    uuid: Uuid,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    // Mojang expects UUID without hyphens (simple format)
    let id = uuid.simple().to_string();
    let url = format!(
        "https://sessionserver.mojang.com/session/minecraft/profile/{}?unsigned=false",
        id
    );

    let client = Client::new();
    let res = client.get(&url).send().await?.error_for_status()?;
    let profile: Profile = res.json().await?;

    // Find the 'textures' property
    let prop = profile
        .properties
        .into_iter()
        .find(|p| p.name == "textures")
        .ok_or("Textures property not found")?;

    let texture_value = prop.value;
    let signature = prop.signature.unwrap_or_default();

    Ok((texture_value, signature))
}
