use std::{collections::HashMap, sync::Arc};
use thecrown_common::{crypto::*, nats::NatsClient};
use thecrown_protocol::{GameServerPacket, GameServerSpecs, McServerPacket};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct State {
    pub nats_client: Arc<NatsClient>,
    pub servers: Arc<Mutex<HashMap<String, Arc<ServerContainer>>>>, // RwLock
    pub game_servers: Arc<Mutex<HashMap<String, Arc<GameServer>>>>, // RwLock
    pub players_game_servers: Arc<Mutex<HashMap<String, Arc<GameServer>>>>,

    // <Token, AuthData>
    pub auth_tokens: Arc<Mutex<HashMap<String, AuthData>>>,
}

impl State {
    pub fn new(nats_client: Arc<NatsClient>) -> Self {
        Self {
            nats_client,
            servers: Arc::new(Mutex::new(HashMap::new())),
            auth_tokens: Arc::new(Mutex::new(HashMap::new())),
            game_servers: Arc::new(Mutex::new(HashMap::new())),
            players_game_servers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn register_game_server(&self, id: String, server: ()) {}

    pub async fn register_container_server(
        &self,
        server_name: String,
        address: String,
        port: u16,
    ) -> Arc<ServerContainer> {
        let server = ServerContainer {
            nats_client: self.nats_client.clone(),
            server_name: server_name.clone(),
            address,
            port,
        };
        let container_server = Arc::new(server);
        {
            let mut map = self.servers.lock().await;
            map.insert(server_name, container_server.clone());
        }

        container_server
    }

    // Prepares the authentication for a player that needs to join a given game name
    pub async fn gen_auth_for_player_game_server(
        &self,
        username: String,
        server: String,
        game_server: String,
    ) -> String {
        // Auth cookie
        let auth_data = AuthData {
            server,
            game_server,
            username,
        };
        let cookie = &self.gen_auth_token(auth_data).await;

        cookie.to_string()
    }

    pub async fn init_game_server(&self, server_spec: &GameServerSpecs, server_container: Arc<ServerContainer>) {
        let server = GameServer {
            nats_client: self.nats_client.clone(),
            server_container,
            server_name: server_spec.name.clone(),
        };
        let server = Arc::new(server);
        {
            let mut map = self.game_servers.lock().await;
            map.insert(server_spec.name.clone(), server.clone());
        }
    }

    pub async fn try_auth_user(&self, username: &str, server: &str, token: &str) -> Option<String> {
        let game_server = {
            let mut map = self.auth_tokens.lock().await;
            let res = match map.get(token) {
                None => None,
                Some(value) => {
                    if value.username == username && value.server == server {
                        Some(value.game_server.clone())
                    } else {
                        None
                    }
                }
            };
            map.remove(token);
            res
        };

        if let Some(ref game_server_name) = game_server {
            let game_server = {
                let map = self.game_servers.lock().await;
                map.get(game_server_name).cloned()
            };
            if let Some(game_server) = game_server {
                let mut map = self.players_game_servers.lock().await;
                map.insert(username.to_string(), game_server);
            }
        }

        game_server
    }

    pub async fn whisper_command(&self, sender: String, target: String, message: String) -> bool {
        let res = {
            let map = self.players_game_servers.lock().await;
            map.get(&target).cloned()
        };
        
        if let Some(target_game_server) = res {
            let packet = McServerPacket::WhisperCommand { sender, target, message };
            target_game_server.server_container.publish(packet).await;
            true
        } else {
            false
        }
    }

    pub async fn gen_auth_token(&self, auth_data: AuthData) -> String {
        log::debug!("Creating token for {}", auth_data.username);

        let token = random_token();
        let mut map = self.auth_tokens.lock().await;
        map.insert(token.clone(), auth_data);
        token
    }
}

#[derive(Debug)]
pub struct ServerContainer {
    pub nats_client: Arc<NatsClient>,
    pub server_name: String,
    pub address: String,
    pub port: u16,
}

impl ServerContainer {
    pub async fn publish(&self, message: McServerPacket) {
        let subject = format!("mcserver.{}", self.server_name);
        self.nats_client
            .publish_with_subject(subject, &message)
            .await;
    }
}

#[derive(Debug)]
pub struct GameServer {
    pub nats_client: Arc<NatsClient>,
    pub server_container: Arc<ServerContainer>,
    pub server_name: String,
}

impl GameServer {
    pub async fn publish(&self, message: GameServerPacket) {
        let subject = format!("gameserver.{}", self.server_name);
        self.nats_client
            .publish_with_subject(subject, &message)
            .await;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthData {
    pub server: String,
    pub game_server: String,
    pub username: String,
}

