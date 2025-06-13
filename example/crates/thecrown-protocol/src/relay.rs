use serde_derive::{Deserialize, Serialize};
use crate::ProtocolPacket;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum RelayPacket {
    /* Server container server -> Relay */
    RegisterServer { server_name: String, address: String, port: u16 },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub struct GameServerSpecs {
    pub name: String,
    pub server_type: GameServerType,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum GameServerType {
    Parkour,
    Lobby
}

impl ProtocolPacket for RelayPacket {
    fn get_nats_subject() -> &'static str {
        "relay"
    }
}