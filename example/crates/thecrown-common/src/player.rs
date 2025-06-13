use thecrown_protocol::TransferPacketData;
use minestom::Player;
use minestom::transfer::TransferPacket;
use minestom::cookie::StoreCookiePacket;

pub const COOKIE_AUTH: &'static str = "auth";

pub trait Transferable {
    fn transfer(&self, data: TransferPacketData);
}

impl Transferable for Player {
    fn transfer(&self, data: TransferPacketData) {
        // Set cookie
        let pckt = StoreCookiePacket::new(COOKIE_AUTH, data.cookie);
        let _ = self.send_packet(&pckt);

        // Send transfer packet
        let pckt = TransferPacket::new(data.address, data.port);
        let _ = self.send_packet(&pckt);

        // Disconnect ?
    }
}