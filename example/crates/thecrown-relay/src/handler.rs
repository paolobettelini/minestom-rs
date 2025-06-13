use chrono::{NaiveDateTime, Utc};
use futures::StreamExt;
use thecrown_protocol::GameServerType;
use thecrown_protocol::McServerPacket;
use crate::State;
use thecrown_protocol::GameServerSpecs;
use thecrown_protocol::RelayPacket;

type PacketType = RelayPacket;

pub async fn handle_msg(state: &State, msg: PacketType) -> Option<PacketType> {
    match msg {
        RelayPacket::RegisterServer { server_name, address, port } => {
            log::info!("Server registered: {server_name} {address}:{port}");
            
            let server_container = state.register_container_server(server_name, address, port).await;

            let servers = vec![
                GameServerSpecs {
                    name: String::from("lobby1"),
                    server_type: GameServerType::Lobby,
                },
                GameServerSpecs {
                    name: String::from("parkour1"),
                    server_type: GameServerType::Parkour,
                }
            ];
            let message = McServerPacket::StartGameServers { servers };
            server_container.publish(message).await;

            None
        }
        _ => None,
    }
}