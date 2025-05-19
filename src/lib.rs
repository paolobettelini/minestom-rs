pub mod block;
pub mod command;
pub mod coordinate;
pub mod entity;
pub mod error;
pub mod event;
pub mod instance;
pub mod jni_env;
pub mod jni_utils;
pub mod server;
pub mod sound;
pub mod text;
pub mod attribute;

pub use error::MinestomError;
pub type Result<T> = std::result::Result<T, MinestomError>;
pub use block::Block;
pub use coordinate::{Pos, Position};
pub use server::MinestomServer;
pub use sound::{Sound, SoundEvent, Source};
pub use text::Component;
pub use attribute::{Attribute, AttributeInstance};

// Re-export commonly used types
use crate::event::CALLBACKS;
use crate::jni_utils::JavaObject;
pub use command::Command;
pub use entity::Player;
pub use event::player::{AsyncPlayerConfigurationEvent, PlayerMoveEvent, PlayerSpawnEvent, PlayerDisconnectEvent};
pub use event::server::ServerListPingEvent;
pub use event::Event;
pub use instance::InstanceContainer;
use jni::objects::{JObject, JString};
use jni::sys::{jlong, jobject, JNIEnv};
use log::{debug, error};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::panic::{self, AssertUnwindSafe};
use std::sync::RwLock;

/// Type alias for event constructor functions that create event instances from JavaObjects
pub type EventConstructor = fn(JavaObject) -> Box<dyn Event>;

/// A registry that maps Java class names to event constructor functions.
/// This allows dynamic instantiation of event types without hardcoded match statements.
pub static EVENT_REGISTRY: Lazy<RwLock<HashMap<String, EventConstructor>>> = Lazy::new(|| {
    let mut registry = HashMap::new();

    // Register built-in event types
    register_event_type::<PlayerSpawnEvent>(&mut registry);
    register_event_type::<AsyncPlayerConfigurationEvent>(&mut registry);
    register_event_type::<PlayerMoveEvent>(&mut registry);
    register_event_type::<ServerListPingEvent>(&mut registry);
    register_event_type::<PlayerDisconnectEvent>(&mut registry);

    RwLock::new(registry)
});

/// Helper function to register an event type in the registry.
fn register_event_type<E: Event + 'static>(registry: &mut HashMap<String, EventConstructor>) {
    let class_name = E::java_class_name().replace("/", ".");

    // Constructor function that creates a new instance of the event
    let constructor: EventConstructor = |java_obj| Box::new(E::new(java_obj));

    registry.insert(class_name, constructor);
}

/// Register a new event type with the global registry.
/// This can be called from anywhere to extend the supported event types.
pub fn register_event<E: Event + 'static>() {
    let class_name = E::java_class_name().replace("/", ".");

    // Constructor function that creates a new instance of the event
    let constructor: EventConstructor = |java_obj| Box::new(E::new(java_obj));

    EVENT_REGISTRY
        .write()
        .unwrap()
        .insert(class_name, constructor);
}

/// Initialize the JVM and required Minestom classes.
/// This must be called before using any other Minestom functionality.
///
/// # Arguments
/// * `jar_path` - Path to the Minestom JAR file
///
/// # Returns
/// Returns a `MinestomServer` instance that can be used to manage the server.
///
/// # Example
/// ```rust,no_run
/// let server = minestom::init()?;
/// let instance_manager = server.instance_manager()?;
/// let instance = instance_manager.create_instance_container()?;
/// server.start("0.0.0.0", 25565)?;
/// ```
pub fn init() -> Result<MinestomServer> {
    MinestomServer::new()
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_org_example_ConsumerCallback_invokeNativeCallback(
    env: *mut JNIEnv,
    _this: jobject,
    callback_id: jlong,
    event: jobject,
) {
    debug!("Native callback invoked with id: {}", callback_id);

    // Catch any panic to prevent unwinding into Java
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        // Convert the raw JNIEnv pointer to a safe JNIEnv wrapper
        let env_wrapper = match unsafe { jni::JNIEnv::from_raw(env) } {
            Ok(env) => env,
            Err(e) => {
                error!("Failed to get JNIEnv: {}", e);
                return;
            }
        };

        // Create a mutable reference to the environment
        let mut env = env_wrapper;

        // Create a frame to automatically delete local references when we're done
        let _frame = match env.push_local_frame(64) {
            Ok(frame) => frame,
            Err(e) => {
                error!("Failed to create local frame: {}", e);
                return;
            }
        };

        // Safely create a JObject from the raw jobject and ensure it's valid
        let event_obj = unsafe { JObject::from_raw(event) };
        if event_obj.is_null() {
            error!("Event object is null");
            return;
        }

        // Get the class using the event object
        let event_class = match env.get_object_class(&event_obj) {
            Ok(class) => class,
            Err(e) => {
                error!("Failed to get event class: {}", e);
                return;
            }
        };

        // Get class name using getName() method
        let class_name_obj =
            match env.call_method(&event_class, "getName", "()Ljava/lang/String;", &[]) {
                Ok(obj) => obj,
                Err(e) => {
                    error!("Failed to call getName(): {}", e);
                    return;
                }
            };

        let class_name: JString = match class_name_obj.l() {
            Ok(s) => s.into(),
            Err(e) => {
                error!("Failed to convert class name to JString: {}", e);
                return;
            }
        };

        let class_name_str: String = match env.get_string(&class_name) {
            Ok(s) => s.into(),
            Err(e) => {
                error!("Failed to convert JString to String: {}", e);
                return;
            }
        };

        debug!("Event class name: {}", class_name_str);

        // Get the callback from our global map
        let callback = {
            let callbacks = CALLBACKS.read();
            match callbacks.get(&(callback_id as u64)) {
                Some(callback) => callback.clone(),
                None => {
                    error!("No callback found for id: {}", callback_id);
                    return;
                }
            }
        };

        // Create a global reference to ensure the event object stays alive
        let global_event = match env.new_global_ref(&event_obj) {
            Ok(global) => global,
            Err(e) => {
                error!("Failed to create global reference: {}", e);
                return;
            }
        };

        // Create a JavaObject from the global reference
        let java_obj = JavaObject::new(global_event);

        // Get the event constructor from the registry
        let event = {
            let registry = EVENT_REGISTRY.read().unwrap();
            match registry.get(&class_name_str) {
                Some(constructor) => constructor(java_obj),
                None => {
                    error!("No event constructor found for: {}", class_name_str);
                    return;
                }
            }
        };

        debug!("Created event object, calling callback...");

        // Call the callback with the event
        if let Err(e) = callback(&*event) {
            error!("Error in event callback: {}", e);
        } else {
            debug!("Successfully called event callback");
        }

        // The frame will be automatically popped when _frame is dropped
    }));

    if let Err(e) = result {
        error!("Panic occurred in native callback: {:?}", e);
    }

    debug!("Native callback completed");
}
