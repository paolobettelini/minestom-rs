use crate::commands::SpawnCommand;
use log::info;
use minestom::MinestomServer;
use minestom::{
    command::{Command, CommandContext},
    component,
    entity::GameMode,
    event::player::{AsyncPlayerConfigurationEvent, PlayerSpawnEvent},
};
use minestom_rs as minestom;

pub struct LobbyMap {
    pub anvil_path: String,
    pub spawn_x: f64,
    pub spawn_y: f64,
    pub spawn_z: f64,
    pub spawn_yaw: f32,
    pub spawn_pitch: f32,
}

pub async fn run_server() -> minestom::Result<()> {
    init_logging();

    let lobby1 = LobbyMap {
        anvil_path: "/home/paolo/Desktop/github/minestom-rs/example-server/anvil/lobby1".to_string(),
        spawn_x: -79.5,
        spawn_y: 153.0,
        spawn_z: -11.5,
        spawn_yaw: -90.0,
        spawn_pitch: 0.0,
    };

    let lobby2 = LobbyMap {
        anvil_path: "/home/paolo/Desktop/github/minestom-rs/example-server/anvil/hub4".to_string(),
        spawn_x: 1817.5,
        spawn_y: 41.0,
        spawn_z: 1044.5,
        spawn_yaw: 90.0,
        spawn_pitch: 0.0,
    };

    let map = lobby2;

    let minecraft_server = MinestomServer::new()?;
    let instance_manager = minecraft_server.instance_manager()?;
    let instance = instance_manager.create_instance_container()?;
    instance.load_anvil_world(map.anvil_path)?;

    // Register commands
    let command_manager = minecraft_server.command_manager()?;
    command_manager.register(SpawnCommand::new(
        map.spawn_x,
        map.spawn_y,
        map.spawn_z,
        map.spawn_yaw,
        map.spawn_pitch,
    ))?;

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
            player.teleport(map.spawn_x, map.spawn_y, map.spawn_z, map.spawn_yaw, map.spawn_pitch)?;
            player.set_allow_flying(true)?;
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
        .filter_level(log::LevelFilter::Debug)
        .format_timestamp(Some(env_logger::fmt::TimestampPrecision::Millis))
        .init();
}
