use thecrown_protocol::TransferPacketData;
use minestom::Player;
use minestom::cookie::CookieStorePacket;
use minestom::transfer::TransferPacket;

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