use crate::commands::SpawnCommand;
use crate::magic_values::*;
use crate::maps::LobbyMap2;
use crate::maps::map::LobbyMap;
use log::info;
use minestom::MinestomServer;
use minestom::{
    attribute::Attribute,
    command::{Command, CommandContext},
    component,
    entity::GameMode,
    event::player::{AsyncPlayerConfigurationEvent, PlayerSpawnEvent},
};
use minestom_rs as minestom;

pub async fn run_server() -> minestom::Result<()> {
    init_logging();

    let lobby2 = LobbyMap2;

    let map = lobby2;

    let minecraft_server = MinestomServer::new()?;
    let instance_manager = minecraft_server.instance_manager()?;
    let instance = instance_manager.create_instance_container()?;
    instance.load_anvil_world(map.anvil_path())?;

    // Register commands
    let command_manager = minecraft_server.command_manager()?;
    command_manager.register(SpawnCommand::new(map))?;

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
        }

        Ok(())
    })?;

    let welcome_instance = instance.clone();
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

            let (x, y, z, yaw, pitch) = map.spawn_coordinate();
            player.teleport(x, y, z, yaw, pitch)?;
            player.set_allow_flying(true)?;

            // https://minecraft.wiki/w/Attribute#Modifiers
            let scale = distribution(AVG_SCALE, MIN_SCALE, MAX_SCALE);
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
