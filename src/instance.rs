use crate::block::Block;
use crate::coordinate::Position;
use crate::entity::Player;
use crate::event::EventHandler;
use crate::jni_utils::{get_env, JavaObject, JniValue};
use crate::MinestomError;
use crate::Result;
use jni::objects::JValue;
use jni::objects::{JObject, JObjectArray};
use log::{debug, error, info};
use std::path::Path;

pub struct InstanceManager {
    inner: JavaObject,
}

/// A container for a Minecraft world instance.
/// This type can be cloned to create multiple references to the same instance.
#[derive(Clone)]
pub struct InstanceContainer {
    inner: JavaObject,
}

impl InstanceManager {
    pub(crate) fn new(inner: JavaObject) -> Self {
        Self { inner }
    }

    pub fn create_instance_container(&self) -> Result<InstanceContainer> {
        let result = self.inner.call_object_method(
            "createInstanceContainer",
            "()Lnet/minestom/server/instance/InstanceContainer;",
            &[],
        )?;

        Ok(InstanceContainer::new(result))
    }

    pub fn get_instance(&self, unique_id: i32) -> Result<Option<InstanceContainer>> {
        let result = self.inner.call_object_method(
            "getInstance",
            "(I)Lnet/minestom/server/instance/Instance;",
            &[JniValue::Int(unique_id)],
        )?;

        let obj = result.as_obj()?;
        if obj.is_null() {
            Ok(None)
        } else {
            Ok(Some(InstanceContainer::new(result)))
        }
    }
}

impl InstanceContainer {
    pub(crate) fn new(inner: JavaObject) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> Result<JObject<'_>> {
        self.inner.as_obj()
    }

    /// Loads an Anvil world into this instance using the Common class implementation.
    ///
    /// # Arguments
    /// * `path` - Path to the Anvil world directory
    pub fn load_anvil_world(&self, path: impl AsRef<Path>) -> Result<()> {
        let mut env = get_env()?;
        let path_str = path.as_ref().to_str().ok_or(MinestomError::InvalidPath)?;

        info!("Loading world from Anvil at {}", path_str);

        // Convert path to Java string
        let path_jstring = env.new_string(path_str)?;

        // Get the instance object
        let instance_obj = self.inner()?;

        // Call Common.loadAnvil
        env.call_static_method(
            "org/example/Common",
            "loadAnvil",
            "(Lnet/minestom/server/instance/InstanceContainer;Ljava/lang/String;)V",
            &[JValue::Object(&instance_obj), JValue::Object(&path_jstring)],
        )?;

        Ok(())
    }

    pub fn get_players(&self) -> Result<Vec<Player>> {
        let result =
            self.inner
                .call_object_method("getPlayers", "()Ljava/util/Collection;", &[])?;

        let mut env = get_env()?;
        let result_obj = result.as_obj()?;
        let array = env.call_method(result_obj, "toArray", "()[Ljava/lang/Object;", &[])?;

        let array = array.l()?;
        let array = JObjectArray::from(array);
        let length = env.get_array_length(&array)?;
        let mut players = Vec::with_capacity(length as usize);

        for i in 0..length {
            let player = env.get_object_array_element(&array, i)?;
            players.push(Player::new(JavaObject::from_env(&mut env, player)?));
        }

        Ok(players)
    }

    pub fn get_chunk(&self, chunk_x: i32, chunk_z: i32) -> Result<bool> {
        let result = self.inner.call_bool_method(
            "loadChunk",
            "(II)Z",
            &[JniValue::Int(chunk_x), JniValue::Int(chunk_z)],
        )?;
        Ok(result)
    }

    pub fn load_chunk(&self, chunk_x: i32, chunk_z: i32) -> Result<()> {
        // Call loadChunk which returns a CompletableFuture
        let result = self.inner.call_object_method(
            "loadChunk",
            "(II)Ljava/util/concurrent/CompletableFuture;",
            &[JniValue::Int(chunk_x), JniValue::Int(chunk_z)],
        )?;

        // Call join() on the CompletableFuture to wait for it to complete
        let mut env = get_env()?;
        let future_obj = result.as_obj()?;
        env.call_method(future_obj, "join", "()Ljava/lang/Object;", &[])?;

        Ok(())
    }

    pub fn unload_chunk(&self, chunk_x: i32, chunk_z: i32) -> Result<()> {
        self.inner.call_void_method(
            "unloadChunk",
            "(II)V",
            &[JniValue::Int(chunk_x), JniValue::Int(chunk_z)],
        )
    }

    pub fn get_spawn_position(&self) -> Result<Position> {
        let result = self.inner.call_object_method(
            "getSpawnLocation",
            "()Lnet/minestom/server/coordinate/Pos;",
            &[],
        )?;

        let mut env = get_env()?;
        let pos = result.as_obj()?;

        let x = env.call_method(&pos, "x", "()D", &[])?.d()?;
        let y = env.call_method(&pos, "y", "()D", &[])?.d()?;
        let z = env.call_method(&pos, "z", "()D", &[])?.d()?;

        Ok(Position::new(x, y, z))
    }

    pub fn set_spawn_position(&self, position: &Position) -> Result<()> {
        let mut env = get_env()?;
        let pos_class = env.find_class("net/minestom/server/coordinate/Pos")?;
        let pos = env.new_object(
            pos_class,
            "(DDD)V",
            &[
                JniValue::Double(position.x).as_jvalue(),
                JniValue::Double(position.y).as_jvalue(),
                JniValue::Double(position.z).as_jvalue(),
            ],
        )?;

        self.inner.call_void_method(
            "setSpawnLocation",
            "(Lnet/minestom/server/coordinate/Pos;)V",
            &[JniValue::Object(pos)],
        )
    }

    /// Sets this instance as the default spawning instance for all players.
    /// This should be called before starting the server.
    pub fn set_as_default_spawn_instance(&self) -> Result<()> {
        let mut env = get_env()?;

        // First, let's try to list the available methods on ConnectionManager to debug the issue
        debug!("Getting ConnectionManager class");
        let connection_manager_class =
            env.find_class("net/minestom/server/network/ConnectionManager")?;

        debug!("Finding MinecraftServer class");
        let minecraft_server = env.find_class("net/minestom/server/MinecraftServer")?;

        // Get the ConnectionManager
        debug!("Getting ConnectionManager from MinecraftServer");
        let connection_manager = env.call_static_method(
            minecraft_server,
            "getConnectionManager",
            "()Lnet/minestom/server/network/ConnectionManager;",
            &[],
        )?;

        debug!("Got connection manager");
        let connection_manager_obj = connection_manager.l()?;
        if connection_manager_obj.is_null() {
            error!("ConnectionManager is null!");
            return Err(MinestomError::EventError(
                "ConnectionManager is null".to_string(),
            ));
        }

        // Get the instance object
        debug!("Getting instance object");
        let instance_obj = self.inner()?;

        // Attempt different method names that might exist in the ConnectionManager
        // Try "setSpawningInstance" instead of "setDefaultInstance"
        debug!("Attempting to call setSpawningInstance on ConnectionManager");
        match env.call_method(
            &connection_manager_obj,
            "setSpawningInstance",
            "(Lnet/minestom/server/instance/Instance;)V",
            &[jni::objects::JValue::Object(&instance_obj)],
        ) {
            Ok(_) => {
                debug!("Successfully called setSpawningInstance");
                debug!("Successfully set default instance");
                return Ok(());
            }
            Err(e) => {
                debug!(
                    "Method setSpawningInstance not found: {}. Trying next method...",
                    e
                );
            }
        }

        // Try "setDefaultSpawningInstance"
        debug!("Attempting to call setDefaultSpawningInstance on ConnectionManager");
        match env.call_method(
            &connection_manager_obj,
            "setDefaultSpawningInstance",
            "(Lnet/minestom/server/instance/Instance;)V",
            &[jni::objects::JValue::Object(&instance_obj)],
        ) {
            Ok(_) => {
                debug!("Successfully called setDefaultSpawningInstance");
                debug!("Successfully set default instance");
                return Ok(());
            }
            Err(e) => {
                debug!(
                    "Method setDefaultSpawningInstance not found: {}. Trying next method...",
                    e
                );
            }
        }

        // Fall back to a direct approach - call a static method on the server class
        // or try other methods that might work
        error!("Could not find a method to set the default instance");
        Err(MinestomError::EventError(
            "Could not find a method to set the default instance".to_string(),
        ))
    }

    /// Sets a block at the specified coordinates.
    ///
    /// # Arguments
    /// * `x` - The x coordinate
    /// * `y` - The y coordinate
    /// * `z` - The z coordinate
    /// * `block` - The block to set
    pub fn set_block(&self, x: i32, y: i32, z: i32, block: Block) -> Result<()> {
        let block_obj = block.to_java_block()?;

        self.inner.call_void_method(
            "setBlock",
            "(IIILnet/minestom/server/instance/block/Block;)V",
            &[
                JniValue::Int(x),
                JniValue::Int(y),
                JniValue::Int(z),
                JniValue::Object(block_obj.as_obj()?),
            ],
        )
    }

    /// Sets the time rate of this instance.
    /// The time rate represents how fast time passes in the instance.
    ///
    /// # Arguments
    /// * `rate` - The time rate (default value is 1)
    pub fn set_time_rate(&self, rate: i32) -> Result<()> {
        self.inner
            .call_void_method("setTimeRate", "(I)V", &[JniValue::Int(rate)])
    }

    /// Gets the event node for this instance.
    /// This can be used to register event listeners specific to this instance.
    pub fn event_node(&self) -> Result<EventHandler> {
        let result = self.inner.call_object_method(
            "eventNode",
            "()Lnet/minestom/server/instance/Instance;",
            &[],
        )?;
        Ok(EventHandler::new(result))
    }
}
