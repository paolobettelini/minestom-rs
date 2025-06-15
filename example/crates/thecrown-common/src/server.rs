use std::{
    any::{Any, TypeId},
    sync::Arc,
};

use minestom::{AsyncPlayerConfigurationEvent, MinestomServer};

pub trait Server: Send + Sync {
    fn init(&self, minecraft_server: &MinestomServer) -> minestom::Result<()>;
    fn init_player(
        &self,
        minecraft_server: &MinestomServer,
        config_event: &AsyncPlayerConfigurationEvent,
    ) -> minestom::Result<()>;

    fn type_id(&self) -> TypeId
    where
        Self: 'static,
    {
        TypeId::of::<Self>()
    }
}

impl dyn Server {
    /// Try to borrow this trait‑object as `&T`. Returns `None` if the
    /// concrete type isn’t `T`.
    pub fn downcast_ref<T: Server + 'static>(&self) -> Option<&T> {
        if self.type_id() == TypeId::of::<T>() {
            // SAFETY: we just confirmed via type_id that the data really is a T.
            Some(unsafe { &*(self as *const dyn Server as *const T) })
        } else {
            None
        }
    }
}

pub trait ArcServerDowncast {
    fn downcast_ref<T: Server + 'static>(&self) -> Option<&T>;
}

impl ArcServerDowncast for Arc<Box<dyn Server + Send + Sync>> {
    fn downcast_ref<T: Server + 'static>(&self) -> Option<&T> {
        let srv_obj: &dyn Server = &***self;
        srv_obj.downcast_ref::<T>()
    }
}
