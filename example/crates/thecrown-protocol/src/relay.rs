use crate::ProtocolPacket;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum RelayPacket {
    /* Server container -> Relay */
    RegisterServer {
        server_name: String,
        address: String,
        port: u16,
    },
    /* Auth -> Relay - When the player wants to join the network */
    PlayerWantsToJoin {
        username: String, /* uuid */
    },
    /* Relay -> Auth - When the player wants to join the netwotk */
    AccomodatePlayer {
        data: AccomodatePlayerData,
    },
    /* Server container -> Relay */
    AuthUserJoin {
        username: String,
        server: String,
        cookie: Vec<u8>,
    },
    /* Relay -> Server container */
    ServeAuthResult {
        game_server: Option<String>,
    },
    /* Game Server -> Relay */
    WhisperCommand {
        sender: String,
        target: String,
        message: String,
    },
    /* Relay -> Game Server */
    WhisperCommandResponse {
        status: bool,
    },
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
    Lobby,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum AccomodatePlayerData {
    Ban {
        reason: String,
        time_left: Option<u64>,
    },
    Join {
        transfer_data: TransferPacketData,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub struct TransferPacketData {
    pub cookie: Vec<u8>,
    pub address: String,
    pub port: u16,
}

impl ProtocolPacket for RelayPacket {
    fn get_nats_subject() -> &'static str {
        "relay"
    }
}
