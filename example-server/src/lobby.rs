use log::{debug, error, info};
use minestom::{Block, MinestomServer};
use minestom::{
    component,
    entity::GameMode,
    event::{
        Event,
        player::{AsyncPlayerConfigurationEvent, PlayerSpawnEvent},
    },
    sound::{Sound, SoundEvent, Source},
    text::Component,
};
use minestom_rs as minestom;

pub async fn run_server() -> minestom::Result<()> {
    init_logging();

    let anvil_path = "/home/paolo/Desktop/github/minestom-rs/example-server/anvil";
    let (spawn_x, spawn_y, spawn_z) = (-79.5, 153.0, -11.5);
    let (spawn_yaw, spawn_pitch) = (-90.0, 0.0);

    let minecraft_server = MinestomServer::new()?;
    let instance_manager = minecraft_server.instance_manager()?;
    let instance = instance_manager.create_instance_container()?;
    instance.load_anvil_world(anvil_path)?;

    let event_handler = minecraft_server.event_handler()?;
    let spawn_instance = instance.clone();

    event_handler.register_event_listener(
        move |config_event: &AsyncPlayerConfigurationEvent| {
            info!("Setting spawning instance for player");
            config_event.spawn_instance(&spawn_instance)?;

            // Try to get player information
            if let Ok(player) = config_event.player() {
                if let Ok(name) = player.get_username() {
                    info!("Player configured: {}", name);
                }
            }

            Ok(())
        },
    )?;

    let welcome_instance = instance.clone();
    event_handler.register_event_listener(move |spawn_event: &PlayerSpawnEvent| {
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
            player.teleport(spawn_x, spawn_y, spawn_z, spawn_yaw, spawn_pitch)?;
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
