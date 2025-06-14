use crate::{GameServerSpecs, ProtocolPacket, TransferPacketData};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum McServerPacket {
    /* Relay -> Server container */
    StartGameServers {
        servers: Vec<GameServerSpecs>,
    },

    /* Relay -> Gameserver */
    WhisperCommand {
        sender: String,
        target: String,
        message: String,
    },

    /* Relay -> Gameserver */
    ExecuteTransfer {
        username: String,
        transfer: TransferPacketData,
    },
}

impl ProtocolPacket for McServerPacket {
    fn get_nats_subject() -> &'static str {
        // TODO don't implement this trait for this
        panic!("This protocol does not have a specific queue! You need to specify it.");
    }
}
