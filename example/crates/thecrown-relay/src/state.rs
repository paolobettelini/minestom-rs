use std::sync::Arc;
use std::collections::HashMap;
use std::collections::HashSet;
use thecrown_common::nats::NatsClient;
use thecrown_protocol::McServerPacket;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct State {
    pub nats_client: Arc<NatsClient>,
    pub servers: Arc<Mutex<HashMap<String, Arc<ServerContainer>>>>,
    //pub game_servers: Arc<Mutex<HashMap<String, Arc<GameServer>>>>,
}

impl State {
    pub fn new(nats_client: Arc<NatsClient>) -> Self {
        Self {
            nats_client,
            servers: Default::default(),
           // game_servers: Default::default(),
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