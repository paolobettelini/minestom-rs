use serde::Deserialize;

#[derive(Deserialize)]
struct MinecraftProfile {
    id: String,
    name: String,
}

pub async fn get_uuid_from_username(username: &str) -> Option<String> {
    let url = format!("https://api.mojang.com/users/profiles/minecraft/{}", username);
    let response = reqwest::get(&url).await.ok()?;
    let profile: MinecraftProfile = response.json().await.ok()?;

    Some(profile.id)
}