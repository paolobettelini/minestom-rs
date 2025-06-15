use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, RwLock},
};

use minestom::{Player, transfer::TransferPacket};
use thecrown_protocol::TransferPacketData;
use uuid::Uuid;

use crate::server::Server;

pub const COOKIE_AUTH: &'static str = "auth";

pub trait Transferable {
    fn transfer(&self, data: TransferPacketData) -> minestom::Result<()>;
}

impl Transferable for Player {
    fn transfer(&self, data: TransferPacketData) -> minestom::Result<()> {
        let connection = self.get_player_connection()?;

        // Set cookie
        connection.store_cookie(COOKIE_AUTH, &data.cookie)?;

        // Send transfer packet
        let pckt = TransferPacket::new(data.address, data.port);
        let _ = self.send_packet(&pckt);

        connection.disconnect()?;
        Ok(())
    }
}

static GAME_SERVERS: LazyLock<RwLock<HashMap<Uuid, Arc<Box<dyn Server + Send + Sync>>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub trait GetGameServer {
    fn set_server(&self, server: Arc<Box<dyn Server + Send + Sync>>) -> minestom::Result<()>;
    fn get_server(&self) -> Option<Arc<Box<dyn Server + Send + Sync>>>;
}

impl GetGameServer for Player {
    fn set_server(&self, server: Arc<Box<dyn Server + Send + Sync>>) -> minestom::Result<()> {
        let uuid = self.get_uuid()?;
        let mut map = GAME_SERVERS.write().expect("Server map poisoned");
        map.insert(uuid, server);
        Ok(())
    }

    fn get_server(&self) -> Option<Arc<Box<dyn Server + Send + Sync>>> {
        let uuid = match self.get_uuid() {
            Ok(id) => id,
            Err(_) => return None,
        };
        let map = GAME_SERVERS.read().ok()?; // swallow poisoning
        map.get(&uuid).cloned()
    }
}
