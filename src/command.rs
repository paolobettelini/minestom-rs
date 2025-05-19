use crate::jni_utils::{get_env, JavaObject, JniValue, ToJava};
use crate::text::Component;
use crate::Result;
use crate::error::MinestomError;
use jni::objects::{JString, JValue, JObject};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use log::{debug, error};

// Store command callbacks
static COMMAND_CALLBACKS: Lazy<RwLock<HashMap<u64, Arc<dyn Fn(&CommandSender, &CommandContext) -> Result<()> + Send + Sync>>>> = 
    Lazy::new(|| RwLock::new(HashMap::new()));

static NEXT_CALLBACK_ID: AtomicU64 = AtomicU64::new(1);

/// Represents a command that can be registered with the command manager
pub trait Command: Send + Sync + Clone + 'static {
    /// Returns the name of the command
    fn name(&self) -> &str;
    
    /// Returns a list of aliases for the command
    fn aliases(&self) -> Vec<&str> {
        Vec::new()
    }
    
    /// Called when the command is executed
    fn execute(&self, sender: &CommandSender, context: &CommandContext) -> Result<()>;
}

/// Represents the context in which a command is executed
pub struct CommandContext {
    inner: JavaObject,
}

impl CommandContext {
    pub(crate) fn new(inner: JavaObject) -> Self {
        Self { inner }
    }

    /// Gets the command sender
    pub fn sender(&self) -> Result<CommandSender> {
        let mut env = get_env()?;
        let result = self.inner.call_object_method(
            "getSender",
            "()Lnet/minestom/server/command/CommandSender;",
            &[],
        )?;
        Ok(CommandSender::new(JavaObject::from_env(&mut env, result.as_obj()?)?))
    }

    /// Gets the command arguments
    pub fn get_string(&self, name: &str) -> Result<String> {
        let mut env = get_env()?;
        let j_name = name.to_java(&mut env)?;
        let result = self.inner.call_object_method(
            "getString",
            "(Ljava/lang/String;)Ljava/lang/String;",
            &[j_name],
        )?;
        let obj = result.as_obj()?;
        let string_ref = JString::from(obj);
        let jstr = env.get_string(&string_ref)?;
        Ok(jstr.to_string_lossy().into_owned())
    }

    pub fn get_integer(&self, name: &str) -> Result<i32> {
        let mut env = get_env()?;
        let j_name = name.to_java(&mut env)?;
        self.inner.call_int_method(
            "getInteger",
            "(Ljava/lang/String;)I",
            &[j_name],
        )
    }
}

/// Represents an entity that can execute commands
pub struct CommandSender {
    inner: JavaObject,
}

impl CommandSender {
    pub(crate) fn new(inner: JavaObject) -> Self {
        Self { inner }
    }

    /// Returns true if this sender is a player
    pub fn is_player(&self) -> Result<bool> {
        let mut env = get_env()?;
        let class_name = env.get_object_class(self.inner.as_obj()?)?;
        let class_name_str = env.find_class("net/minestom/server/entity/Player")?;
        Ok(env.is_assignable_from(&class_name, &class_name_str)?)
    }

    /// Converts this sender to a player if possible
    pub fn as_player(&self) -> Result<crate::entity::Player> {
        if !self.is_player()? {
            return Err(MinestomError::InvalidPlayer("Sender is not a player".to_string()).into());
        }
        Ok(crate::entity::Player::new(self.inner.clone()))
    }

    /// Sends a message to the command sender
    pub fn send_message(&self, message: &Component) -> Result<()> {
        let mut env = get_env()?;
        let j_message = message.as_jvalue(&mut env)?;
        
        self.inner.call_void_method(
            "sendMessage",
            "(Lnet/kyori/adventure/text/Component;)V",
            &[j_message],
        )
    }

    /// Returns true if the sender has the given permission
    pub fn has_permission(&self, permission: &str) -> Result<bool> {
        let mut env = get_env()?;
        let j_permission = permission.to_java(&mut env)?;
        
        self.inner.call_bool_method(
            "hasPermission",
            "(Ljava/lang/String;)Z",
            &[j_permission],
        )
    }
}

/// Manages command registration and execution
pub struct CommandManager {
    inner: JavaObject,
}

impl CommandManager {
    pub(crate) fn new(inner: JavaObject) -> Self {
        Self { inner }
    }

    /// Registers a new command with the command manager
    pub fn register<T: Command + 'static>(&self, command: T) -> Result<CommandBuilder> {
        let mut env = get_env()?;
        
        // Create the Java command object
        let command_class = env.find_class("net/minestom/server/command/builder/Command")?;
        let j_name = command.name().to_java(&mut env)?;
        
        // Create the aliases array
        let aliases = command.aliases();
        let aliases_array = env.new_object_array(
            aliases.len() as i32,
            "java/lang/String",
            JObject::null(),
        )?;
        for (i, alias) in aliases.iter().enumerate() {
            let j_alias = env.new_string(alias)?;
            env.set_object_array_element(&aliases_array, i as i32, j_alias)?;
        }

        let command_obj = env.new_object(
            command_class,
            "(Ljava/lang/String;[Ljava/lang/String;)V",
            &[j_name.as_jvalue(), JValue::Object(&aliases_array).into()],
        )?;

        // Clone command before moving it
        let command = Arc::new(command.clone());

        // Store the command callback
        let callback_id = NEXT_CALLBACK_ID.fetch_add(1, Ordering::SeqCst);
        let callback = Arc::new(move |sender: &CommandSender, context: &CommandContext| {
            command.execute(sender, context)
        });
        COMMAND_CALLBACKS.write().insert(callback_id, callback);
        
        // Create the command executor
        let executor_class = env.find_class("org/example/CommandExecutorCallback")?;
        let executor = env.new_object(
            executor_class,
            "(J)V",
            &[JValue::Long(callback_id as i64)],
        )?;

        // Add the executor to the command
        env.call_method(
            &command_obj,
            "setDefaultExecutor",
            "(Lnet/minestom/server/command/builder/CommandExecutor;)V",
            &[JValue::Object(&executor)],
        )?;
        
        // Create a global reference for the command object
        let global_command = env.new_global_ref(&command_obj)?;
        
        // Register the command with Minestom
        let local_command = env.new_local_ref(&command_obj)?;
        self.inner.call_void_method(
            "register",
            "(Lnet/minestom/server/command/builder/Command;)V",
            &[JniValue::Object(local_command)],
        )?;
        
        Ok(CommandBuilder::new(JavaObject::new(global_command)))
    }

    /// Unregisters a command by name
    pub fn unregister(&self, name: &str) -> Result<()> {
        let mut env = get_env()?;
        let j_name = name.to_java(&mut env)?;
        
        self.inner.call_void_method(
            "unregister",
            "(Ljava/lang/String;)V",
            &[j_name],
        )
    }
}

/// Builder for configuring commands with arguments and conditions
pub struct CommandBuilder {
    inner: JavaObject,
}

impl CommandBuilder {
    pub(crate) fn new(inner: JavaObject) -> Self {
        Self { inner }
    }

    /// Adds a required string argument to the command
    pub fn add_string_arg(&self, name: &str) -> Result<&Self> {
        let mut env = get_env()?;
        let j_name = name.to_java(&mut env)?;
        
        self.inner.call_void_method(
            "addSyntax",
            "(Lnet/minestom/server/command/builder/arguments/ArgumentString;)Lnet/minestom/server/command/builder/Command;",
            &[j_name],
        )?;
        
        Ok(self)
    }

    /// Adds a required integer argument to the command
    pub fn add_integer_arg(&self, name: &str) -> Result<&Self> {
        let mut env = get_env()?;
        let j_name = name.to_java(&mut env)?;
        
        self.inner.call_void_method(
            "addSyntax",
            "(Lnet/minestom/server/command/builder/arguments/ArgumentInteger;)Lnet/minestom/server/command/builder/Command;",
            &[j_name],
        )?;
        
        Ok(self)
    }

    /// Sets a condition that must be met for the command to execute
    pub fn set_condition<F>(&self, condition: F) -> Result<&Self>
    where
        F: Fn(&CommandSender) -> Result<bool> + Send + Sync + 'static,
    {
        let mut env = get_env()?;
        
        // Create a new callback ID for the condition
        let callback_id = NEXT_CALLBACK_ID.fetch_add(1, Ordering::SeqCst);
        let condition = Arc::new(move |sender: &CommandSender, _context: &CommandContext| {
            match condition(sender) {
                Ok(true) => Ok(()),
                Ok(false) => Ok(()),
                Err(e) => Err(e),
            }
        });
        COMMAND_CALLBACKS.write().insert(callback_id, condition);
        
        // Create the condition executor
        let condition_class = env.find_class("org/example/CommandConditionCallback")?;
        let condition_obj = env.new_object(
            condition_class,
            "(J)V",
            &[JValue::Long(callback_id as i64)],
        )?;

        // Set the condition
        self.inner.call_void_method(
            "setCondition",
            "(Lnet/minestom/server/command/builder/condition/CommandCondition;)Lnet/minestom/server/command/builder/Command;",
            &[JniValue::Object(condition_obj)],
        )?;
        
        Ok(self)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_org_example_CommandExecutorCallback_executeCommand(
    env: *mut jni::sys::JNIEnv,
    _class: jni::objects::JClass,
    callback_id: jni::sys::jlong,
    sender: jni::objects::JObject,
    context: jni::objects::JObject,
) {
    // Catch any panic to prevent unwinding into Java
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
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

        // Create global references to ensure the objects stay alive
        let global_context = match env.new_global_ref(&context) {
            Ok(global) => global,
            Err(e) => {
                error!("Failed to create global reference for context: {}", e);
                return;
            }
        };

        let global_sender = match env.new_global_ref(&sender) {
            Ok(global) => global,
            Err(e) => {
                error!("Failed to create global reference for sender: {}", e);
                return;
            }
        };

        // Create JavaObjects from the global references
        let context_obj = JavaObject::new(global_context);
        let sender_obj = JavaObject::new(global_sender);

        // Create the command context and sender
        let context = CommandContext::new(context_obj);
        let sender = CommandSender::new(sender_obj);

        // Get the callback from our global map
        let callback = {
            let callbacks = COMMAND_CALLBACKS.read();
            match callbacks.get(&(callback_id as u64)) {
                Some(callback) => callback.clone(),
                None => {
                    error!("No callback found for id: {}", callback_id);
                    return;
                }
            }
        };

        debug!("Executing command callback...");

        // Execute the callback
        if let Err(e) = callback(&sender, &context) {
            error!("Error executing command: {}", e);
        }
    }));

    if let Err(e) = result {
        error!("Panic occurred in command callback: {:?}", e);
    }
}
