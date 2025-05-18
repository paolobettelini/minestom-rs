use crate::coordinate::{Pos, Position};
use crate::entity::Player;
use crate::instance::InstanceContainer;
use crate::jni_utils::{get_env, JavaObject, JniValue};
use crate::{MinestomError, Result};
use jni::objects::JString;
use log::{debug, error, info};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

// Re-export event types at the top-level
pub use self::player::{AsyncPlayerConfigurationEvent, PlayerSpawnEvent};

pub(crate) static CALLBACKS: Lazy<
    RwLock<HashMap<u64, Arc<dyn Fn(&dyn Event) -> Result<()> + Send + Sync>>>,
> = Lazy::new(|| RwLock::new(HashMap::new()));

static NEXT_CALLBACK_ID: AtomicU64 = AtomicU64::new(1);

/// Represents Minestom's GlobalEventHandler that can be used to register event listeners.
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

    /// Registers an event listener for a specific event type.
    /// The callback will receive the specific event type directly, not a generic `&dyn Event`.
    ///
    /// # Type Parameters
    /// * `E` - The event type that implements the `Event` trait
    ///
    /// # Arguments
    /// * `callback` - A function that will be called with the specific event type
    pub fn register_event_listener<E, F>(&self, callback: F) -> Result<()>
    where
        E: Event + 'static,
        F: Fn(&E) -> Result<()> + Send + Sync + 'static,
    {
        // Create a wrapper function that will downcast the dyn Event to the specific type
        let wrapper = move |event: &dyn Event| -> Result<()> {
            // Downcast the event to the specific type
            if let Some(e) = event.as_any().downcast_ref::<E>() {
                callback(e)
            } else {
                // This should never happen unless there's a bug in the Java side
                error!("Failed to downcast event to {}", std::any::type_name::<E>());
                Err(MinestomError::EventError(format!(
                    "Failed to downcast event to {}",
                    std::any::type_name::<E>()
                )))
            }
        };

        self.register_event_listener_internal(E::java_class_name(), wrapper)
    }

    /// Internal implementation for registering an event listener.
    /// This is used by the generic `register_event_listener` method.
    fn register_event_listener_internal<F>(&self, event_class_name: &str, callback: F) -> Result<()>
    where
        F: Fn(&dyn Event) -> Result<()> + Send + Sync + 'static,
    {
        let mut env = get_env()?;

        debug!("Finding required classes for {}...", event_class_name);

        // Find needed classes - use more robust class lookups
        let listener_class = match env.find_class("net/minestom/server/event/EventListener") {
            Ok(class) => class,
            Err(e) => {
                error!("Failed to find EventListener class: {}", e);
                return Err(MinestomError::EventError(
                    "Failed to find EventListener class".to_string(),
                ));
            }
        };

        let event_class = match env.find_class(event_class_name) {
            Ok(class) => class,
            Err(e) => {
                error!("Failed to find event class {}: {}", event_class_name, e);
                return Err(MinestomError::EventError(format!(
                    "Failed to find event class {}",
                    event_class_name
                )));
            }
        };

        let callback_class = match env.find_class("org/example/ConsumerCallback") {
            Ok(class) => class,
            Err(e) => {
                error!("Failed to find ConsumerCallback class: {}", e);
                return Err(MinestomError::EventError(
                    "Failed to find ConsumerCallback class".to_string(),
                ));
            }
        };

        debug!("Found all required classes");

        // Store the callback in our global map
        let callback_id = NEXT_CALLBACK_ID.fetch_add(1, Ordering::SeqCst);
        let callback = Arc::new(callback);

        // Insert the callback before creating the Java objects to ensure it's available
        CALLBACKS.write().insert(callback_id, callback);
        info!("Created callback with ID: {}", callback_id);

        // Create our callback instance with the callback ID
        let callback_instance = match env.new_object(
            &callback_class,
            "(J)V",
            &[JniValue::Long(callback_id as i64).as_jvalue()],
        ) {
            Ok(instance) => instance,
            Err(e) => {
                error!("Failed to create callback instance: {}", e);
                // Remove the callback since we failed to create the Java object
                CALLBACKS.write().remove(&callback_id);
                return Err(MinestomError::EventError(
                    "Failed to create callback instance".to_string(),
                ));
            }
        };

        debug!("Created callback instance");

        // Create a global reference for the callback instance to prevent GC
        let callback_global_ref = match env.new_global_ref(callback_instance) {
            Ok(global_ref) => global_ref,
            Err(e) => {
                error!("Failed to create global reference for callback: {}", e);
                CALLBACKS.write().remove(&callback_id);
                return Err(MinestomError::EventError(
                    "Failed to create global reference for callback".to_string(),
                ));
            }
        };
        let callback_obj = callback_global_ref.as_obj();

        // Create the EventListener using the static of() method
        debug!("Creating EventListener...");
        let result = match env.call_static_method(
            &listener_class,
            "of",
            "(Ljava/lang/Class;Ljava/util/function/Consumer;)Lnet/minestom/server/event/EventListener;",
            &[
                jni::objects::JValue::Object(&event_class),
                jni::objects::JValue::Object(&callback_obj),
            ],
        ) {
            Ok(result) => result,
            Err(e) => {
                error!("Failed to create EventListener: {}", e);
                CALLBACKS.write().remove(&callback_id);
                return Err(MinestomError::EventError("Failed to create EventListener".to_string()));
            }
        };

        let listener = match result.l() {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to get EventListener object: {}", e);
                CALLBACKS.write().remove(&callback_id);
                return Err(MinestomError::EventError(
                    "Failed to get EventListener object".to_string(),
                ));
            }
        };
        debug!("Created event listener");

        // Create a JavaObject from the listener and keep it alive
        let listener = match JavaObject::from_env(&mut env, listener) {
            Ok(obj) => obj,
            Err(e) => {
                error!("Failed to create JavaObject from listener: {}", e);
                CALLBACKS.write().remove(&callback_id);
                return Err(MinestomError::EventError(
                    "Failed to create JavaObject from listener".to_string(),
                ));
            }
        };

        debug!("Adding event listener to event handler...");
        match self.inner.call_void_method(
            "addListener",
            "(Lnet/minestom/server/event/EventListener;)Lnet/minestom/server/event/EventNode;",
            &[JniValue::from(listener.as_obj()?)],
        ) {
            Ok(_) => {
                info!("Event listener added successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to add event listener: {}", e);
                CALLBACKS.write().remove(&callback_id);
                Err(MinestomError::EventError(
                    "Failed to add event listener".to_string(),
                ))
            }
        }
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
}
