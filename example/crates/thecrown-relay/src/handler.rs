use crate::State;
use rand::{Rng, seq::IndexedRandom};
use thecrown_protocol::{
    AccomodatePlayerData, GameServerSpecs, GameServerType, McServerPacket, RelayPacket,
    TransferPacketData,
};

type PacketType = RelayPacket;

pub async fn handle_msg(state: &State, msg: PacketType) -> Option<PacketType> {
    match msg {
        RelayPacket::RegisterServer {
            server_name,
            address,
            port,
        } => {
            log::info!("Server registered: {server_name} {address}:{port}");

            let server_container = state
                .register_container_server(server_name, address, port)
                .await;

            let servers = vec![
                GameServerSpecs {
                    name: String::from("lobby1"),
                    server_type: GameServerType::Lobby,
                },
                GameServerSpecs {
                    name: String::from("lobby2"),
                    server_type: GameServerType::Lobby,
                },
                GameServerSpecs {
                    name: String::from("parkour1"),
                    server_type: GameServerType::Parkour,
                },
            ];
            let message = McServerPacket::StartGameServers { servers };
            server_container.publish(message).await;

            None
        }
        RelayPacket::PlayerWantsToJoin { username } => {
            log::info!("Received join request by {username}");

            // check if banned
            // TODO: two requests are sent to mojang to get the UUID.
            /*if let Some(ban) = state.db_client.get_ban(&username).await {
                let time_left = if let Some(end) = ban.ban_end {
                    let now = Utc::now().timestamp();
                    Some(end.timestamp() - now)
                } else {
                    None
                };

                let response = RelayPacket::AccomodatePlayer {
                    data: AccomodatePlayerData::Ban {
                        reason: ban.ban_reason.to_string(),
                        time_left,
                    },
                };

                return Some(response);
            }

            // get player data
            if let Some(player) = state.db_client.get_player(&username).await {
                // maybe get the type of lobby he was previously in?
            } else {
                // Initialize player to the database
                state.db_client.insert_player(&username).await;
            }

            // Get lobby "hub" for player
            let (game_server, cookie) = state
                .get_lobby_for_player(&username, "hub")
                .await
                .expect("No hub servers found");
            */

            let server = "server1".to_string();

            //let game_servers = vec!["lobby1", "lobby2"];
            let game_servers = vec!["lobby1", "parkour1"];
            let game_server = game_servers.choose(&mut rand::rng()).unwrap().to_string();
            log::info!("Putting into {}", &game_server);

            let address = "127.0.0.1".to_string();
            let port = 25566;
            let cookie = state
                .gen_auth_for_player_game_server(username, server, game_server)
                .await;
            let transfer_data = TransferPacketData {
                cookie: cookie.into(),
                address,
                port,
            };

            let response = RelayPacket::AccomodatePlayer {
                data: AccomodatePlayerData::Join { transfer_data },
            };

            Some(response)
        }
        RelayPacket::AuthUserJoin {
            username,
            server,
            cookie,
        } => {
            let game_server = if let Ok(token) = String::from_utf8(cookie) {
                state.try_auth_user(&username, &server, &token).await
            } else {
                None
            };
            Some(RelayPacket::ServeAuthResult { game_server })
        }
        RelayPacket::WhisperCommand { sender, target, message } => {
            let status = todo!();
            Some(RelayPacket::WhisperCommandResponse { status })
        }
        _ => None,
    }
}
