use crate::Result;
use crate::item::{InventoryHolder, PlayerInventory};
use crate::jni_utils::{JniValue, get_env};
use crate::resource_pack::ResourcePackRequest;

impl crate::entity::Player {
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
