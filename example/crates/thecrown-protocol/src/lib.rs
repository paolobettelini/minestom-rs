use serde::{Deserialize, Serialize};

pub trait ProtocolPacket {
    fn get_nats_subject() -> &'static str;
}

mod relay;
//mod web;
mod mcserver;
mod gameserver;

pub use relay::*;
//pub use web::*;
pub use mcserver::*;
pub use gameserver::*;