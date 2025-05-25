use jni::objects::{JObject, JValue};

use crate::Result;
use crate::entity::PlayerSkin;
use crate::item::{InventoryHolder, PlayerInventory};
use crate::jni_utils::{JniValue, get_env};
use crate::resource_pack::ResourcePackRequest;

/// Trait for anything that knows how to send itself as a packet to a player.

pub trait SendablePacket {
    fn to_java(&self) -> JObject;
}

impl crate::entity::Player {
    /// Sets the player's skin
    pub fn set_skin(&self, skin: &PlayerSkin) -> Result<()> {
        let mut env = get_env()?;
        self.inner.call_void_method(
            "setSkin",
            "(Lnet/minestom/server/entity/PlayerSkin;)V",
            &[skin.inner().as_jvalue(&mut env)?],
        )
    }

    /// Invia un pacchetto particelle al player via JNI
    pub fn send_packet<P: SendablePacket>(&self, packet: &P) -> crate::Result<()> {
        let java_pkt: JObject = packet.to_java();
        let mut env = get_env()?;
        env.call_method(
            self.inner.as_obj()?,
            "sendPacket",
            "(Lnet/minestom/server/network/packet/server/SendablePacket;)V",
            &[JValue::Object((&java_pkt).into())],
        )?;
        Ok(())
    }

    /// Sends resource packs to the player
    pub fn send_resource_packs(&self, request: &ResourcePackRequest) -> Result<()> {
        let mut env = get_env()?;
        let request_obj = request.as_obj().as_obj()?;
        self.inner.call_void_method(
            "sendResourcePacks",
            "(Lnet/kyori/adventure/resource/ResourcePackRequest;)V",
            &[JniValue::Object(request_obj)],
        )
    }

    /// Clears all resource packs from the player
    pub fn clear_resource_packs(&self) -> Result<()> {
        let mut env = get_env()?;
        self.inner
            .call_void_method("clearResourcePacks", "()V", &[])
    }

    /// Refreshes the commands for this player
    pub fn refresh_commands(&self) -> Result<()> {
        self.inner.call_void_method(
            "refreshCommands",
            "()V",
            &[],
        )
    }
}

impl InventoryHolder for crate::entity::Player {
    fn get_inventory(&self) -> Result<PlayerInventory> {
        let mut env = get_env()?;
        let inventory = self.inner.call_object_method(
            "getInventory",
            "()Lnet/minestom/server/inventory/PlayerInventory;",
            &[],
        )?;

        PlayerInventory::from_java(inventory.as_obj()?)
    }
}
 