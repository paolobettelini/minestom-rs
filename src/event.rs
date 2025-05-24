use crate::coordinate::{Pos, Position};
use crate::entity::{Player, PlayerSkin};
use crate::instance::InstanceContainer;
use crate::jni_utils::{JavaObject, JniValue, get_env};
use crate::text::Component;
use crate::{MinestomError, Result};
use jni::objects::{JObject, JObjectArray, JString};
use log::{debug, error};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

// Re-export event types at the top-level
pub use self::player::{AsyncPlayerConfigurationEvent, PlayerSpawnEvent};

pub(crate) static CALLBACKS: Lazy<
    RwLock<HashMap<u64, Arc<dyn Fn(&dyn Event) -> Result<()> + Send + Sync>>>,
> = Lazy::new(|| RwLock::new(HashMap::new()));

pub(crate) static FILTER_CALLBACKS: Lazy<
    RwLock<HashMap<u64, Arc<dyn Fn(&Player) -> bool + Send + Sync>>>,
> = Lazy::new(|| RwLock::new(HashMap::new()));

static NEXT_CALLBACK_ID: AtomicU64 = AtomicU64::new(1);

/// Represents Minestom's EventNode that can be used to register event listeners.
#[derive(Clone)]
pub struct EventNode {
    inner: Arc<JavaObject>,
}

impl EventNode {
    pub(crate) fn from(inner: JavaObject) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }

    /// Creates a new event listener with optional priority
    pub fn listen_with_priority<E: Event + 'static>(
        &self,
        priority: Option<i32>,
        callback: impl Fn(&E) -> Result<()> + Send + Sync + 'static,
    ) -> Result<()> {
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
            MinestomError::EventError(format!(
                "Failed to find event class {}",
                E::java_class_name()
            ))
        })?;

        let callback_class = env
            .find_class("org/example/ConsumerCallback")
            .map_err(|e| {
                error!("Failed to find ConsumerCallback class: {}", e);
                MinestomError::EventError("Failed to find ConsumerCallback class".to_string())
            })?;

        // Store callback
        let callback_id = NEXT_CALLBACK_ID.fetch_add(1, Ordering::SeqCst);
        let callback = Arc::new(wrapper);
        CALLBACKS.write().insert(callback_id, callback);

        // Create callback instance
        let callback_instance = env
            .new_object(
                &callback_class,
                "(J)V",
                &[JniValue::Long(callback_id as i64).as_jvalue()],
            )
            .map_err(|e| {
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
    pub fn listen<E: Event + 'static>(
        &self,
        callback: impl Fn(&E) -> Result<()> + Send + Sync + 'static,
    ) -> Result<()> {
        self.listen_with_priority::<E>(None, callback)
    }

    pub fn listen_async<E, F, Fut>(&self, callback: F) -> Result<()>
    where
        E: Event + Send + Clone + 'static,
        F: Fn(E) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        // Wrap the callback in an Arc so we can clone it inside the sync closure
        let callback = Arc::new(callback);
        // Create a sync callback for listen_with_priority
        let sync_cb = move |e: &E| -> Result<()> {
            let owned = e.clone();
            let cb = callback.clone();
            crate::TOKIO_HANDLE.spawn(async move {
                if let Err(err) = (cb)(owned).await {
                    error!("async handler error: {}", err);
                }
            });
            Ok(())
        };
        // Delegate to the existing sync listener
        self.listen_with_priority::<E>(None, sync_cb)
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

    /// Creates a new event node with a custom condition for player events
    pub fn create_player_filter<F>(name: &str, filter: F) -> Result<Self> 
    where
        F: Fn(&Player) -> bool + Send + Sync + 'static,
    {
        let mut env = get_env()?;

        // Store the filter callback
        let callback_id = NEXT_CALLBACK_ID.fetch_add(1, Ordering::SeqCst);
        let filter = Arc::new(filter);
        FILTER_CALLBACKS.write().insert(callback_id, filter);

        // Create the Java predicate
        let predicate_class = env.find_class("org/example/PredicateCallback")?;
        let predicate = env.new_object(
            predicate_class,
            "(J)V",
            &[(callback_id as i64).into()],
        )?;
        let predicate_global = env.new_global_ref(predicate)?;

        // Get the PLAYER filter
        let event_filter_class = env.find_class("net/minestom/server/event/EventFilter")?;
        let player_filter = env.get_static_field(
            event_filter_class,
            "PLAYER",
            "Lnet/minestom/server/event/EventFilter;",
        )?.l()?;
        let player_filter_global = env.new_global_ref(player_filter)?;

        // Create name string
        let name_jstring = env.new_string(name)?;

        // Call the static value method to create the EventNode
        let event_node_class = env.find_class("net/minestom/server/event/EventNode")?;
        let result = env.call_static_method(
            event_node_class,
            "value",
            "(Ljava/lang/String;Lnet/minestom/server/event/EventFilter;Ljava/util/function/Predicate;)Lnet/minestom/server/event/EventNode;",
            &[
                (&name_jstring).into(),
                (&*player_filter_global).into(),
                (&*predicate_global).into(),
            ],
        )?;

        let node = result.l()?;
        let node_global = env.new_global_ref(node)?;
        Ok(Self::from(JavaObject::new(node_global)))
    }

    /// Adds a child node to this event node.
    /// Children take the condition of their parent and are able to append to it.
    pub fn add_child(&self, child: &EventNode) -> Result<EventNode> {
        let mut env = get_env()?;
        let result = self.inner.call_object_method(
            "addChild",
            "(Lnet/minestom/server/event/EventNode;)Lnet/minestom/server/event/EventNode;",
            &[JniValue::Object(child.inner.as_obj()?).into()],
        )?;
        Ok(EventNode::from(result))
    }
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
    #[derive(Clone)]
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

    /// Event fired when a player sends a chat message.
    pub struct PlayerChatEvent {
        inner: JavaObject,
    }

    impl PlayerChatEvent {
        /// Gets the player who sent the message.
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

        /// Gets the raw message sent by the player.
        pub fn raw_message(&self) -> Result<String> {
            let mut env = get_env()?;
            let result = self.inner.call_object_method(
                "getRawMessage",
                "()Ljava/lang/String;",
                &[],
            )?;
            let java_obj = result.as_obj()?;
            let jstring = JString::from(java_obj);
            let java_str = env.get_string(&jstring)?;
            Ok(java_str.to_string_lossy().into_owned())
        }

        /// Gets the formatted message that will be displayed in chat.
        pub fn formatted_message(&self) -> Result<Component> {
            let mut env = get_env()?;
            let result = self.inner.call_object_method(
                "getFormattedMessage",
                "()Lnet/kyori/adventure/text/Component;",
                &[],
            )?;
            let java_obj = JavaObject::from_env(&mut env, result.as_obj()?)?;
            Ok(Component::from_java_object(java_obj))
        }

        /// Sets the formatted message that will be displayed in chat.
        pub fn set_formatted_message(&self, message: &Component) -> Result<()> {
            let mut env = get_env()?;
            self.inner.call_void_method(
                "setFormattedMessage",
                "(Lnet/kyori/adventure/text/Component;)V",
                &[message.as_jvalue(&mut env)?],
            )
        }

        /// Gets whether the event is cancelled.
        pub fn is_cancelled(&self) -> Result<bool> {
            self.inner.call_bool_method("isCancelled", "()Z", &[])
        }

        /// Sets whether the event is cancelled.
        pub fn set_cancelled(&self, cancelled: bool) -> Result<()> {
            self.inner.call_void_method(
                "setCancelled",
                "(Z)V",
                &[JniValue::Bool(cancelled)]
            )
        }

        /// Gets the recipients of this chat message.
        pub fn recipients(&self) -> Result<Vec<Player>> {
            let mut env = get_env()?;
            let result = self.inner.call_object_method(
                "getRecipients",
                "()Ljava/util/Collection;",
                &[],
            )?;
            let collection = result.as_obj()?;
            
            // Convert to array
            let array = env.call_method(
                collection,
                "toArray",
                "()[Ljava/lang/Object;",
                &[],
            )?.l()?;

            let array = JObjectArray::from(array);
            let length = env.get_array_length(&array)?;
            let mut players = Vec::with_capacity(length as usize);

            for i in 0..length {
                let player_obj = env.get_object_array_element(&array, i)?;
                players.push(Player::new(JavaObject::from_env(&mut env, player_obj)?));
            }

            Ok(players)
        }
    }

    impl Event for PlayerChatEvent {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn java_class_name() -> &'static str {
            "net/minestom/server/event/player/PlayerChatEvent"
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
    use crate::jni_utils::ToJava;
    use crate::text::Component;

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
pub use self::ping::ResponseData;
pub use self::player::PlayerDisconnectEvent;
pub use self::server::ServerListPingEvent;

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_org_example_PredicateCallback_testPlayer(
    env: *mut jni::sys::JNIEnv,
    _this: jni::objects::JObject,
    callback_id: jni::sys::jlong,
    player_obj: jni::objects::JObject,
) -> jni::sys::jboolean {
    unsafe {
        // Convert the raw JNIEnv pointer to a safe wrapper
        let mut env = match jni::JNIEnv::from_raw(env) {
            Ok(env) => env,
            Err(_) => return 0,
        };

        // Create a frame to manage local references
        let _frame = match env.push_local_frame(16) {
            Ok(frame) => frame,
            Err(_) => return 0,
        };

        // Get the filter callback from our global map
        let filter = {
            let callbacks = FILTER_CALLBACKS.read();
            match callbacks.get(&(callback_id as u64)) {
                Some(callback) => callback.clone(),
                None => return 0,
            }
        };

        // Create a Player instance from the JObject
        let player_obj_global = match env.new_global_ref(player_obj) {
            Ok(global) => global,
            Err(_) => return 0,
        };

        let player = Player::new(JavaObject::new(player_obj_global));

        // Execute the filter callback
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if filter(&player) {
                1
            } else {
                0
            }
        })) {
            Ok(1) => 1,
            _ => 0,
        }
    }
}
