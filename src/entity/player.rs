use crate::resource_pack::ResourcePackRequest;
use crate::Result;
use crate::jni_utils::{get_env, JniValue};

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
        self.inner.call_void_method(
            "clearResourcePacks",
            "()V",
            &[],
        )
    }
} 