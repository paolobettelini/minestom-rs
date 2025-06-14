use std::sync::Arc;
use std::collections::HashMap;
use std::collections::HashSet;
use thecrown_common::nats::NatsClient;
use thecrown_protocol::McServerPacket;
use tokio::sync::Mutex;
use thecrown_common::crypto::*;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct State {
    pub nats_client: Arc<NatsClient>,
    pub servers: Arc<Mutex<HashMap<String, Arc<ServerContainer>>>>, // RwLock
    //pub game_servers: Arc<Mutex<HashMap<String, Arc<GameServer>>>>,

    // <Token, AuthData>
    pub auth_tokens: Arc<Mutex<HashMap<String, AuthData>>>,
}

impl State {
    pub fn new(nats_client: Arc<NatsClient>) -> Self {
        Self {
            nats_client,
            servers: Arc::new(Mutex::new(HashMap::new())),
            auth_tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn register_game_server(&self, id: String, server: ()) {

    }

    pub async fn register_container_server(&self, server_name: String, address: String, port: u16) -> Arc<ServerContainer> {
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
    pub async fn gen_auth_for_player_game_server(&self, username: String, server: String, game_server: String) -> String {
        // Auth cookie
        let auth_data = AuthData {
            server,
            game_server,
            username,
        };
        let cookie = &self.gen_auth_token(auth_data).await;

        cookie.to_string()
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

        // TODO update count and register player to that game server

        game_server
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
        self.nats_client.publish_with_subject(subject, &message).await;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthData {
    pub server: String,
    pub game_server: String,
    pub username: String,
}