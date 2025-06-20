use reqwest::Client;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub properties: Vec<Property>,
}

#[derive(Deserialize)]
pub struct Property {
    pub name: String,
    pub value: String,
    pub signature: Option<String>,
}

#[derive(Deserialize)]
pub struct TexturePayload {
    pub textures: Textures,
}

#[derive(Deserialize)]
pub struct Textures {
    pub skin: Option<SkinInfo>,
}

#[derive(Deserialize)]
pub struct SkinInfo {
    pub url: String,
    pub metadata: Option<Metadata>,
}

#[derive(Deserialize)]
pub struct Metadata {
    pub model: Option<String>,
}

pub async fn get_uuid_from_username(username: &str) -> Option<String> {
    let url = format!(
        "https://api.mojang.com/users/profiles/minecraft/{}",
        username
    );
    let response = reqwest::get(&url).await.ok()?;
    let profile: Profile = response.json().await.ok()?;

    Some(profile.id)
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
