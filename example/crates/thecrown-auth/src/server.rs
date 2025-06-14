use log::info;
use minestom::{
    self, MinestomServer, ServerListPingEvent, TOKIO_HANDLE, component,
    event::player::AsyncPlayerConfigurationEvent,
};
use std::sync::Arc;
use thecrown_common::{nats::NatsClient, player::*, text};
use thecrown_protocol::{AccomodatePlayerData::*, RelayPacket};

pub async fn run_server() -> anyhow::Result<()> {
    init_logging();

    let minecraft_server = MinestomServer::new()?;
    let event_handler = minecraft_server.event_handler()?;

    let nats_url = String::from("127.0.0.1:4222");
    let nats_client = Arc::new(NatsClient::new(nats_url).await?);

    event_handler.listen(move |config_event: &AsyncPlayerConfigurationEvent| {
        // Try to get player information
        if let Ok(player) = config_event.player() {
            if let Ok(username) = player.get_username() {
                // Send PlayerWantsToJoin to Relay
                let packet = RelayPacket::PlayerWantsToJoin { username };
                let response = TOKIO_HANDLE.block_on(async { nats_client.request(&packet).await });

                if let Some(RelayPacket::AccomodatePlayer { data }) = response {
                    match data {
                        Ban { reason, time_left } => {
                            let component = text::create_ban_message(&reason, time_left);
                            player.kick(&component)?;
                        }
                        Join { transfer_data } => {
                            player.transfer(transfer_data)?;
                        }
                    }
                }
            }
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
