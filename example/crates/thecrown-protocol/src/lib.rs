pub trait ProtocolPacket {
    fn get_nats_subject() -> &'static str;
}

mod relay;
//mod web;
mod gameserver;
mod mcserver;

pub use relay::*;
//pub use web::*;
pub use gameserver::*;
pub use mcserver::*;
