use crate::ProtocolPacket;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum GameServerPacket {
    Dummy,
}

impl ProtocolPacket for GameServerPacket {
    fn get_nats_subject() -> &'static str {
        // TODO don't implement this trait for this
        panic!("This protocol does not have a specific queue! You need to specify it.");
    }
}
