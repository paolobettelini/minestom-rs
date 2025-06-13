use serde_derive::{Deserialize, Serialize};
use crate::ProtocolPacket;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum WebPacket {
    GenerateAuthToken { username: String }, // The username of the player who wants to auth.
    ServeAuthToken { token: String },
}

impl ProtocolPacket for WebPacket {
    fn get_nats_subject() -> &'static str {
        "web"
    }
}