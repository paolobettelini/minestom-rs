use crate::coordinate::{Pos, Position};
use crate::entity::{Player, PlayerSkin};
use crate::instance::InstanceContainer;
use crate::jni_utils::{get_env, JavaObject, JniValue, ToJava};
use crate::{MinestomError, Result};
use jni::objects::JString;
use log::{debug, error, info};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use crate::text::Component;

// Re-export event types at the top-level
pub use self::player::{AsyncPlayerConfigurationEvent, PlayerSpawnEvent};

pub(crate) static CALLBACKS: Lazy<
    RwLock<HashMap<u64, Arc<dyn Fn(&dyn Event) -> Result<()> + Send + Sync>>>,
> = Lazy::new(|| RwLock::new(HashMap::new()));

static NEXT_CALLBACK_ID: AtomicU64 = AtomicU64::new(1);

/// Represents Minestom's EventNode that can be used to register event listeners.
/// This is the root event node that receives all server events.
#[derive(Clone)]
pub struct EventHandler {
    inner: Arc<JavaObject>,
}

/// Trait implemented by all Minestom events.
///
/// This trait is used to provide a common interface for all events and enable
/// dynamic dispatch in event handlers. It also implements `Any` to allow downcasting
/// to concrete event types.
pub trait Event: Any {
    /// Returns a reference to self as `Any` to enable downcasting.
    fn as_any(&self) -> &dyn Any;

    /// Returns the Java class name of this event.
    /// This is used to register event listeners.
    fn java_class_name() -> &'static str
    where
        Self: Sized;

    /// Creates a new instance of this event from a JavaObject.
    /// This is used by the event registry to create events dynamically.
    fn new(java_obj: JavaObject) -> Self
    where
        Self: Sized;
}

impl EventHandler {
    pub(crate) fn new(inner: JavaObject) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }

    /// Creates a new event listener with optional priority
    pub fn listen_with_priority<E: Event + 'static>(&self, priority: Option<i32>, callback: impl Fn(&E) -> Result<()> + Send + Sync + 'static) -> Result<()> {
        // Create wrapper that handles priority if set
        let wrapper = move |event: &dyn Event| -> Result<()> {
            if let Some(e) = event.as_any().downcast_ref::<E>() {
                callback(e)
            } else {
                error!("Failed to downcast event to {}", std::any::type_name::<E>());
                Err(MinestomError::EventError(format!(
                    "Failed to downcast event to {}",
                    std::any::type_name::<E>()
                )))
            }
        };

        let mut env = get_env()?;

        // Find event class
        let event_class = env.find_class(E::java_class_name()).map_err(|e| {
            error!("Failed to find event class {}: {}", E::java_class_name(), e);
            MinestomError::EventError(format!("Failed to find event class {}", E::java_class_name()))
        })?;

        let callback_class = env.find_class("org/example/ConsumerCallback").map_err(|e| {
            error!("Failed to find ConsumerCallback class: {}", e);
            MinestomError::EventError("Failed to find ConsumerCallback class".to_string())
        })?;

        // Store callback
        let callback_id = NEXT_CALLBACK_ID.fetch_add(1, Ordering::SeqCst);
        let callback = Arc::new(wrapper);
        CALLBACKS.write().insert(callback_id, callback);

        // Create callback instance
        let callback_instance = env.new_object(
            &callback_class,
            "(J)V",
            &[JniValue::Long(callback_id as i64).as_jvalue()],
        ).map_err(|e| {
            error!("Failed to create callback instance: {}", e);
            CALLBACKS.write().remove(&callback_id);
            MinestomError::EventError("Failed to create callback instance".to_string())
        })?;

        let callback_global_ref = env.new_global_ref(callback_instance).map_err(|e| {
            error!("Failed to create global reference for callback: {}", e);
            CALLBACKS.write().remove(&callback_id);
            MinestomError::EventError("Failed to create global reference for callback".to_string())
        })?;

        // Set priority if specified
        if let Some(priority) = priority {
            self.set_priority(priority)?;
        }

        // Add listener using the global reference directly
        self.inner.call_void_method(
            "addListener",
            "(Ljava/lang/Class;Ljava/util/function/Consumer;)Lnet/minestom/server/event/EventNode;",
            &[
                JniValue::Object(event_class.into()).into(),
                JniValue::Object(JavaObject::global_to_local(&callback_global_ref)?).into(),
            ],
        )?;

        Ok(())
    }

    /// Convenience method to register a listener without priority
    pub fn listen<E: Event + 'static>(&self, callback: impl Fn(&E) -> Result<()> + Send + Sync + 'static) -> Result<()> {
        self.listen_with_priority::<E>(None, callback)
    }

    /// Gets the priority of this event handler.
    pub fn get_priority(&self) -> Result<i32> {
        self.inner.call_int_method("getPriority", "()I", &[])
    }

    /// Sets the priority of this event handler.
    /// Higher priority handlers receive events first.
    pub fn set_priority(&self, priority: i32) -> Result<()> {
        self.inner.call_void_method(
            "setPriority",
            "(I)Lnet/minestom/server/event/EventNode;",
            &[JniValue::Int(priority)],
        )?;
        Ok(())
    }
}

pub mod player {
    use super::*;

    /// Event fired when a player spawns in the server.
    pub struct PlayerSpawnEvent {
        inner: JavaObject,
    }

    impl PlayerSpawnEvent {
        /// Gets the player that spawned.
        pub fn player(&self) -> Result<Player> {
            let mut env = get_env()?;
            let event_obj = self.inner.as_obj()?;

            let result = env.call_method(
                event_obj,
                "getPlayer",
                "()Lnet/minestom/server/entity/Player;",
                &[],
            )?;

            let player_obj = result.l()?;
            Ok(Player::new(JavaObject::new(
                env.new_global_ref(player_obj)?,
            )))
        }
    }

    impl Event for PlayerSpawnEvent {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn java_class_name() -> &'static str {
            "net/minestom/server/event/player/PlayerSpawnEvent"
        }

        fn new(inner: JavaObject) -> Self {
            Self { inner }
        }
    }

    /// Event fired when a player's configuration is being set up.
    /// This event is fired before the player spawns and can be used to set
    /// the player's initial state.
    pub struct AsyncPlayerConfigurationEvent {
        pub inner: JavaObject,
    }

    impl AsyncPlayerConfigurationEvent {
        /// Sets the instance where the player will spawn.
        pub fn spawn_instance(&self, instance: &InstanceContainer) -> Result<()> {
            let mut env = get_env()?;

            debug!("Setting spawning instance for player configuration...");

            // Get the instance's Java object
            let instance_obj = instance.inner()?;

            // Create a local frame to manage references
            let _frame = env.push_local_frame(16)?;

            // Call setSpawningInstance on the event
            let result = self.inner.call_void_method(
                "setSpawningInstance",
                "(Lnet/minestom/server/instance/Instance;)V",
                &[JniValue::from(instance_obj)],
            );

            // Check for exceptions even if the call succeeded
            if env.exception_check()? {
                let exception = env.exception_occurred()?;
                env.exception_clear()?;

                // Get exception details
                let message = if let Ok(msg) =
                    env.call_method(&exception, "getMessage", "()Ljava/lang/String;", &[])
                {
                    if let Ok(msg_obj) = msg.l() {
                        let jstring = JString::from(msg_obj);
                        let msg_str = env.get_string(&jstring);
                        match msg_str {
                            Ok(s) => s.to_string_lossy().into_owned(),
                            Err(_) => "Unknown error".to_string(),
                        }
                    } else {
                        "Unknown error".to_string()
                    }
                } else {
                    "Unknown error".to_string()
                };

                error!(
                    "Exception occurred while setting spawning instance: {}",
                    message
                );
                return Err(MinestomError::EventError(format!(
                    "Failed to set spawning instance: {}",
                    message
                )));
            }

            // Handle the actual method call result
            match result {
                Ok(_) => {
                    debug!("Successfully set spawning instance");
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to set spawning instance: {}", e);
                    Err(MinestomError::EventError(
                        "Failed to set spawning instance".to_string(),
                    ))
                }
            }
        }

        /// Gets the player being configured.
        pub fn player(&self) -> Result<Player> {
            let mut env = get_env()?;
            let result = self.inner.call_object_method(
                "getPlayer",
                "()Lnet/minestom/server/entity/Player;",
                &[],
            )?;
            let java_obj = JavaObject::from_env(&mut env, result.as_obj()?)?;
            Ok(Player::new(java_obj))
        }

        /// Returns true if this is the first time the player is in the configuration phase.
        pub fn is_first_config(&self) -> Result<bool> {
            debug!("Checking if this is first configuration...");
            self.inner
                .call_bool_method("isFirstConfiguration", "()Z", &[])
        }
    }

    impl Event for AsyncPlayerConfigurationEvent {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn java_class_name() -> &'static str {
            "net/minestom/server/event/player/AsyncPlayerConfigurationEvent"
        }

        fn new(inner: JavaObject) -> Self {
            Self { inner }
        }
    }

    pub struct PlayerMoveEvent {
        inner: JavaObject,
    }

    impl PlayerMoveEvent {
        pub fn player(&self) -> Result<Player> {
            let mut env = get_env()?;
            let result = self.inner.call_object_method(
                "getPlayer",
                "()Lnet/minestom/server/entity/Player;",
                &[],
            )?;
            let java_obj = JavaObject::from_env(&mut env, result.as_obj()?)?;
            Ok(Player::new(java_obj))
        }

        pub fn new_position(&self) -> Result<Position> {
            let mut env = get_env()?;
            let result = self.inner.call_object_method(
                "getNewPosition",
                "()Lnet/minestom/server/coordinate/Pos;",
                &[],
            )?;
            let pos = Pos::new(JavaObject::from_env(&mut env, result.as_obj()?)?);
            pos.to_position()
        }

        pub fn cancel(&mut self) -> Result<()> {
            self.inner
                .call_void_method("setCancelled", "(Z)V", &[JniValue::Bool(true)])
        }
    }

    impl Event for PlayerMoveEvent {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn java_class_name() -> &'static str {
            "net/minestom/server/event/player/PlayerMoveEvent"
        }

        fn new(inner: JavaObject) -> Self {
            Self { inner }
        }
    }

    pub struct PlayerDisconnectEvent {
        inner: JavaObject,
    }

    impl PlayerDisconnectEvent {
        /// Gets the player that disconnected.
        pub fn player(&self) -> Result<Player> {
            let mut env = get_env()?;
            let result = self.inner.call_object_method(
                "getPlayer",
                "()Lnet/minestom/server/entity/Player;",
                &[],
            )?;
            let java_obj = JavaObject::from_env(&mut env, result.as_obj()?)?;
            Ok(Player::new(java_obj))
        }
    }

    impl Event for PlayerDisconnectEvent {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn java_class_name() -> &'static str {
            "net/minestom/server/event/player/PlayerDisconnectEvent"
        }

        fn new(inner: JavaObject) -> Self {
            Self { inner }
        }
    }

    /// Event fired when a player's skin is being initialized.
    pub struct PlayerSkinInitEvent {
        inner: JavaObject,
    }

    impl PlayerSkinInitEvent {
        /// Gets the player whose skin is being initialized.
        pub fn player(&self) -> Result<Player> {
            let mut env = get_env()?;
            let result = self.inner.call_object_method(
                "getPlayer",
                "()Lnet/minestom/server/entity/Player;",
                &[],
            )?;
            let java_obj = JavaObject::from_env(&mut env, result.as_obj()?)?;
            Ok(Player::new(java_obj))
        }

        /// Sets the player's skin.
        pub fn set_skin(&self, skin: &PlayerSkin) -> Result<()> {
            let mut env = get_env()?;
            self.inner.call_void_method(
                "setSkin",
                "(Lnet/minestom/server/entity/PlayerSkin;)V",
                &[skin.inner().as_jvalue(&mut env)?],
            )
        }
    }

    impl Event for PlayerSkinInitEvent {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn java_class_name() -> &'static str {
            "net/minestom/server/event/player/PlayerSkinInitEvent"
        }

        fn new(inner: JavaObject) -> Self {
            Self { inner }
        }
    }
}

pub mod server {
    use super::*;

    pub struct ServerListPingEvent {
        inner: JavaObject,
    }

    impl ServerListPingEvent {
        pub fn get_response_data(&self) -> Result<ResponseData> {
            let mut env = get_env()?;
            let response_data = env.call_method(
                &self.inner.as_obj()?,
                "getResponseData",
                "()Lnet/minestom/server/ping/ResponseData;",
                &[],
            )?;
            
            Ok(ResponseData {
                inner: JavaObject::from_env(&mut env, response_data.l()?)?,
            })
        }
    }

    impl Event for ServerListPingEvent {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn java_class_name() -> &'static str {
            "net/minestom/server/event/server/ServerListPingEvent"
        }

        fn new(inner: JavaObject) -> Self {
            Self { inner }
        }
    }
}

pub mod ping {
    use super::*;
    use crate::text::Component;
    use crate::jni_utils::ToJava;

    pub struct ResponseData {
        pub(crate) inner: JavaObject,
    }

    impl ResponseData {
        pub fn set_name(&self, name: &str) -> Result<()> {
            let mut env = get_env()?;
            let name = name.to_java(&mut env)?;
            env.call_method(
                &self.inner.as_obj()?,
                "setName",
                "(Ljava/lang/String;)V",
                &[name.as_jvalue()],
            )?;
            Ok(())
        }

        pub fn set_version(&self, version: &str) -> Result<()> {
            let mut env = get_env()?;
            let version = version.to_java(&mut env)?;
            env.call_method(
                &self.inner.as_obj()?,
                "setVersion",
                "(Ljava/lang/String;)V",
                &[version.as_jvalue()],
            )?;
            Ok(())
        }

        pub fn set_protocol(&self, protocol: i32) -> Result<()> {
            let mut env = get_env()?;
            env.call_method(
                &self.inner.as_obj()?,
                "setProtocol",
                "(I)V",
                &[JniValue::Int(protocol).as_jvalue()],
            )?;
            Ok(())
        }

        pub fn set_max_player(&self, max_player: i32) -> Result<()> {
            let mut env = get_env()?;
            env.call_method(
                &self.inner.as_obj()?,
                "setMaxPlayer",
                "(I)V",
                &[JniValue::Int(max_player).as_jvalue()],
            )?;
            Ok(())
        }

        pub fn set_online(&self, online: i32) -> Result<()> {
            let mut env = get_env()?;
            env.call_method(
                &self.inner.as_obj()?,
                "setOnline",
                "(I)V",
                &[JniValue::Int(online).as_jvalue()],
            )?;
            Ok(())
        }

        pub fn set_description(&self, description: &Component) -> Result<()> {
            let mut env = get_env()?;
            let description = description.as_jvalue(&mut env)?;
            env.call_method(
                &self.inner.as_obj()?,
                "setDescription",
                "(Lnet/kyori/adventure/text/Component;)V",
                &[description.as_jvalue()],
            )?;
            Ok(())
        }

        pub fn set_favicon(&self, favicon: &str) -> Result<()> {
            let mut env = get_env()?;
            let favicon = favicon.to_java(&mut env)?;
            env.call_method(
                &self.inner.as_obj()?,
                "setFavicon",
                "(Ljava/lang/String;)V",
                &[favicon.as_jvalue()],
            )?;
            Ok(())
        }
    }
}

// Re-export at the top level
pub use self::server::ServerListPingEvent;
pub use self::ping::ResponseData;
pub use self::player::PlayerDisconnectEvent;
