use crate::commands::SpawnCommand;
use crate::magic_values::*;
use crate::maps::LobbyMap2;
use crate::maps::map::LobbyMap;
use crate::mojang::get_skin_and_signature;
use log::info;
use minestom::MinestomServer;
use minestom_rs as minestom;
use minestom_rs::ServerListPingEvent;
use minestom_rs::TOKIO_HANDLE;
use minestom_rs::entity::PlayerSkin;
use minestom_rs::{
    attribute::Attribute,
    command::{Command, CommandContext},
    component,
    entity::GameMode,
    event::player::{
        AsyncPlayerConfigurationEvent, PlayerMoveEvent, PlayerSkinInitEvent, PlayerSpawnEvent,
    },
    item::{InventoryHolder, ItemStack},
    material::Material,
    resource_pack::{ResourcePackInfo, ResourcePackRequest, ResourcePackRequestBuilder},
};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::{Mutex, RwLock};
use uuid::Uuid;

pub static PLAYER_SERVER: Lazy<Arc<RwLock<HashMap<Uuid, String>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

//pub static SERVERS: Lazy<Arc<Mutex<HashMap<Uuid, Arc<Server>>>>> = Lazy::new(|| {
//    Arc::new(Mutex::new(HashMap::new()))
//});

pub async fn run_server() -> minestom::Result<()> {
    init_logging();

    let minecraft_server = MinestomServer::new()?;
    let instance_manager = minecraft_server.instance_manager()?;

    let map = LobbyMap2::new(&instance_manager)?;
    let instance = map.instance();

    // Register commands
    let command_manager = minecraft_server.command_manager()?;
    command_manager.register(SpawnCommand::new(map.clone()))?;

    let event_handler = minecraft_server.event_handler()?;
    let spawn_instance = instance.clone();

    event_handler.listen(move |config_event: &AsyncPlayerConfigurationEvent| {
        info!("Setting spawning instance for player");
        config_event.spawn_instance(&spawn_instance)?;

        // Try to get player information
        if let Ok(player) = config_event.player() {
            if let Ok(name) = player.get_username() {
                info!("Player configured: {}", name);
            }

            // Send resource pack
            let uuid = uuid::Uuid::new_v4();
            let url = "http://127.0.0.1:8080/resourcepack.zip";
            let hash = "123456";

            let pack_info = ResourcePackInfo::new(uuid, url, hash)?;
            let request = ResourcePackRequestBuilder::new()?
                .packs(pack_info)?
                .prompt(&component!("Please accept the resource pack").gold())?
                .required(true)?
                .build()?;

            player.send_resource_packs(&request)?;
        }

        Ok(())
    })?;

    let map_clone = map.clone();
    event_handler.listen(move |spawn_event: &PlayerSpawnEvent| {
        info!("Player spawn event triggered");
        if let Ok(player) = spawn_event.player() {
            let username = player.get_username()?;

            let welcome_msg = component!("Welcome to the server, {}!", username)
                .gold()
                .bold();
            let info_msg = component!("Enjoy your adventure!").green().italic();
            let message = welcome_msg.chain_newline(info_msg);

            player.send_message(&message)?;
            player.set_game_mode(GameMode::Adventure)?;

            let (x, y, z, yaw, pitch) = map_clone.spawn_coordinate();
            player.teleport(x, y, z, yaw, pitch)?;
            player.set_allow_flying(true)?;

            // https://minecraft.wiki/w/Attribute#Modifiers
            let scale = distribution(AVG_SCALE, MIN_SCALE, MAX_SCALE);
            //let scale = 15.0;
            info!("Setting player scale to {}", scale);
            player
                .get_attribute(Attribute::Scale)?
                .set_base_value(scale)?;
            player
                .get_attribute(Attribute::JumpStrength)?
                .set_base_value(jump_strength_scale(scale))?;
            player
                .get_attribute(Attribute::StepHeight)?
                .set_base_value(step_height_scale(scale))?;

            // Create a diamond sombrero
            let sombrero = ItemStack::of(Material::Diamond)?
                .with_amount(1)?
                .with_custom_model_data("piano")?;

            // Get player's inventory and set the helmet
            let inventory = player.get_inventory()?;
            inventory.set_helmet(&sombrero)?;
        }
        Ok(())
    })?;

    event_handler.listen(move |event: &ServerListPingEvent| {
        let response_data = event.get_response_data()?;

        response_data.set_online(-1)?;
        response_data.set_max_player(i32::MAX)?;
        response_data.set_description(&component!("Henlo").red())?;
        response_data.set_favicon(&crate::favicon::random_image())?;

        Ok(())
    })?;

    event_handler.listen(move |skin_event: &PlayerSkinInitEvent| {
        info!("Player skin init event triggered");
        if let Ok(player) = skin_event.player() {
            if let Ok(uuid) = player.get_uuid() {
                let texture = "ewogICJ0aW1lc3RhbXAiIDogMTc0Nzc3NjEwNjQ3OSwKICAicHJvZmlsZUlkIiA6ICI2YTg0ODY0ZDkwYmM0ZjY3YjdiZjI3YTdmZjA2NTc5ZCIsCiAgInByb2ZpbGVOYW1lIiA6ICJIeXBlUGF1bCIsCiAgInNpZ25hdHVyZVJlcXVpcmVkIiA6IHRydWUsCiAgInRleHR1cmVzIiA6IHsKICAgICJTS0lOIiA6IHsKICAgICAgInVybCIgOiAiaHR0cDovL3RleHR1cmVzLm1pbmVjcmFmdC5uZXQvdGV4dHVyZS9jNWY0MDhjZmVkM2I3YzI5MjlkZmI3YTkxY2RlZGU3NmI0NzFhOTAzODgwNmIyZGI1YWMyNGU5NDY1MmQ3Y2Y5IgogICAgfQogIH0KfQ==";
                let signature = "bUaJDR96Rvg+E3KP6ErWEK4K+SIGDA/3Cd5wAFkTZ6XBS8FtEMcKglRrvg8BCe4Djs3oFSK4dSx4tOI3uGxjmf8sAkKG7YvBie43A65PJQHZxsIqWVTb3n22wOW4SRsv0vW9hTyf2UxuLNpquldTXtUmRq6e+c4eR0qFBwA3cyv9zJbQwD3oonlluvc9mgv9qOZ85LJJXybW5mbHqVO16/S/z6m3url3qGMBER8hf99Pl8MeVBgT2khO/rEhLKHkEKRxjvqgQLRUztumTGvgLG8egU6sNWqTKmlpf4IdDqQ7KnVQdxnGa40c24hKjEne+TybH9JZIUZYctDxEcs+TWi6x6upwAQhjd11BgGCzJ8WmVnuqFxEeCZl53QynxkMzMYxjVVRqVZ1k+1C74loNmjRJ2TNzanpIqvAO+xSPp8SC7FWc/toxa98svGTLx9FCyr0Dz5W6S173WKRaUlMIhPATx/2PTNNrclVUcBfzj0TBO9cs350yq9n00jbyOXodHDlIS6zGT0RGpHHqNW/bnUSCqx/SVystXbAsUvAMf6K8kC4IRgPaawm8Hg79Frv8DXcRfyhi7WapuWV+TvxnZTw2k1sbRNF+cwh0fi2PAq3XiSRN5aIE5RZZfhgpFG/XvndhAOasg2qmrEmIhN6UIqGHE6nNS15EASW+4TjxLU=";
                //let (skin_url, signature) = get_skin_and_signature(uuid).await?;
                let (texture, signature) = TOKIO_HANDLE.block_on(async {
                    get_skin_and_signature(uuid).await
                }).unwrap();
                let skin = PlayerSkin::create(&texture, &signature)?;
                skin_event.set_skin(&skin)?;
            }
        }
        Ok(())
    })?;

    info!("Starting server on 0.0.0.0:25565...");
    minecraft_server.start("0.0.0.0", 25565)?;

    info!("Server is now listening for connections!");
    info!("Press Ctrl+C to stop the server");

    // Keep the main thread alive
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C");

    info!("Shutting down server...");

    Ok(())
}

fn init_logging() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp(Some(env_logger::fmt::TimestampPrecision::Millis))
        .init();
}
