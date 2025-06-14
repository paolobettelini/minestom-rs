use crate::Result;
use crate::error::MinestomError;
use crate::jni_utils::{JavaObject, JniValue, ToJava, get_env};
use crate::text::Component;
use jni::objects::{JObject, JString, JValue};
use log::{debug, error};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

// Store command callbacks
static COMMAND_CALLBACKS: Lazy<
    RwLock<HashMap<u64, Arc<dyn Fn(&CommandSender, &CommandContext) -> Result<()> + Send + Sync>>>,
> = Lazy::new(|| RwLock::new(HashMap::new()));

// Store condition callbacks
static CONDITION_CALLBACKS: Lazy<
    RwLock<HashMap<u64, Arc<dyn Fn(&CommandSender) -> Result<bool> + Send + Sync>>>,
> = Lazy::new(|| RwLock::new(HashMap::new()));

// Store suggestion callbacks
static SUGGESTION_CALLBACKS: Lazy<
    RwLock<HashMap<u64, Arc<dyn Fn(&CommandSender, &CommandContext, &mut Suggestion) -> Result<()> + Send + Sync>>>,
> = Lazy::new(|| RwLock::new(HashMap::new()));

static NEXT_CALLBACK_ID: AtomicU64 = AtomicU64::new(1);

/// Represents a command argument
pub struct Argument {
    inner: JavaObject,
}

impl Argument {
    pub(crate) fn new(inner: JavaObject) -> Self {
        Self { inner }
    }

    /// Sets a suggestion callback for this argument
    pub fn set_suggestion_callback<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(&CommandSender, &CommandContext, &mut Suggestion) -> Result<()> + Send + Sync + 'static,
    {
        let mut env = get_env()?;

        // Store the suggestion callback
        let callback_id = NEXT_CALLBACK_ID.fetch_add(1, Ordering::SeqCst);
        let callback = Arc::new(callback);
        SUGGESTION_CALLBACKS.write().insert(callback_id, callback);

        // Create the suggestion callback executor
        let callback_class = env.find_class("rust/minestom/SuggestionCallback")?;
        let callback_obj =
            env.new_object(callback_class, "(J)V", &[JValue::Long(callback_id as i64)])?;

        // Set the suggestion callback
        self.inner.call_object_method(
            "setSuggestionCallback",
            "(Lnet/minestom/server/command/builder/suggestion/SuggestionCallback;)Lnet/minestom/server/command/builder/arguments/Argument;",
            &[JniValue::Object(callback_obj)],
        )?;

        Ok(())
    }

    pub(crate) fn inner(&self) -> &JavaObject {
        &self.inner
    }
}

/// Creates a string argument
pub fn create_string_arg(name: &str) -> Result<Argument> {
    let mut env = get_env()?;
    let j_name = name.to_java(&mut env)?;

    let arg_class = env.find_class("net/minestom/server/command/builder/arguments/ArgumentString")?;
    let arg_obj = env.new_object(arg_class, "(Ljava/lang/String;)V", &[j_name.as_jvalue()])?;

    Ok(Argument::new(JavaObject::from_env(&mut env, arg_obj)?))
}

/// Creates an integer argument
pub fn create_integer_arg(name: &str) -> Result<Argument> {
    let mut env = get_env()?;
    let j_name = name.to_java(&mut env)?;

    let arg_class = env.find_class("net/minestom/server/command/builder/arguments/ArgumentInteger")?;
    let arg_obj = env.new_object(arg_class, "(Ljava/lang/String;)V", &[j_name.as_jvalue()])?;

    Ok(Argument::new(JavaObject::from_env(&mut env, arg_obj)?))
}

/// Creates a player argument
pub fn create_player_arg(name: &str, only_players: bool) -> Result<Argument> {
    let mut env = get_env()?;
    let j_name = name.to_java(&mut env)?;

    let arg_class = env.find_class("net/minestom/server/command/builder/arguments/minecraft/ArgumentEntity")?;
    let arg_obj = env.new_object(arg_class, "(Ljava/lang/String;Z)V", &[j_name.as_jvalue(), JniValue::Bool(false).as_jvalue()])?;

    env.call_method(&arg_obj, "onlyPlayers", "(Z)Lnet/minestom/server/command/builder/arguments/minecraft/ArgumentEntity;", &[JniValue::Bool(only_players).as_jvalue()])?;

    Ok(Argument::new(JavaObject::from_env(&mut env, arg_obj)?))
}

/// Creates a greedy string argument (consumes all remaining text)
pub fn create_greedy_string_arg(name: &str) -> Result<Argument> {
    let mut env = get_env()?;
    let j_name = name.to_java(&mut env)?;

    let arg_class = env.find_class("net/minestom/server/command/builder/arguments/ArgumentStringArray")?;
    let arg_obj = env.new_object(arg_class, "(Ljava/lang/String;)V", &[j_name.as_jvalue()])?;

    Ok(Argument::new(JavaObject::from_env(&mut env, arg_obj)?))
}

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

    /// Gets the command arguments
    pub fn get_string(&self, name: &str) -> Result<String> {
        let mut env = get_env()?;
        let j_name = name.to_java(&mut env)?;
        let result = self.inner.call_object_method(
            "get",
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[j_name],
        )?;
        let obj = result.as_obj()?;
        let string_ref = JString::from(obj);
        let jstr = env.get_string(&string_ref)?;
        Ok(jstr.to_string_lossy().into_owned())
    }

    /// Gets a player argument by name
    pub fn get_player(&self, name: &str) -> Result<EntityFinder> {
        let mut env = get_env()?;
        let j_name = name.to_java(&mut env)?;

        let result = self.inner.call_object_method(
            "get",
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[j_name],
        )?;

        // The result should be an EntityFinder object
        let finder_obj = JavaObject::from_env(&mut env, result.as_obj()?)?;
        Ok(EntityFinder::new(finder_obj))
    }

    /// Gets a string array argument (for greedy strings) and joins them with spaces
    pub fn get_string_array(&self, name: &str) -> Result<Vec<String>> {
        let mut env = get_env()?;
        let j_name = name.to_java(&mut env)?;
        let result = self.inner.call_object_method(
            "get",
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[j_name],
        )?;

        // The result should be a String array
        let array_obj = result.as_obj()?;
        let object_array = jni::objects::JObjectArray::from(array_obj);
        let array_length = env.get_array_length(&object_array)?;

        let mut strings = Vec::new();
        for i in 0..array_length {
            let element = env.get_object_array_element(&object_array, i)?;
            if !element.is_null() {
                let string_ref = JString::from(element);
                let jstr = env.get_string(&string_ref)?;
                strings.push(jstr.to_string_lossy().into_owned());
            }
        }

        Ok(strings)
    }
}

/// Represents an EntityFinder that can find entities
pub struct EntityFinder {
    inner: JavaObject,
}

impl EntityFinder {
    pub(crate) fn new(inner: JavaObject) -> Self {
        Self { inner }
    }

    /// Finds the first player using the given sender
    pub fn find_first_player(&self, sender: &CommandSender) -> Result<crate::entity::Player> {
        let mut env = get_env()?;

        // Call findFirstPlayer() to get the actual Player
        let sender_obj = sender.inner.as_obj()?;
        let player_result = env.call_method(
            self.inner.as_obj()?,
            "findFirstPlayer",
            "(Lnet/minestom/server/command/CommandSender;)Lnet/minestom/server/entity/Player;",
            &[JValue::Object(&sender_obj)],
        )?;

        let player_obj = JavaObject::from_env(&mut env, player_result.l()?)?;
        Ok(crate::entity::Player::new(player_obj))
    }
}

/// Represents a suggestion entry for command completion
pub struct SuggestionEntry {
    inner: JavaObject,
}

impl SuggestionEntry {
    /// Creates a new suggestion entry with the given text
    pub fn new(text: &str) -> Result<Self> {
        let mut env = get_env()?;
        let j_text = text.to_java(&mut env)?;
        
        let entry_class = env.find_class("net/minestom/server/command/builder/suggestion/SuggestionEntry")?;
        let entry_obj = env.new_object(
            entry_class,
            "(Ljava/lang/String;)V",
            &[j_text.as_jvalue()],
        )?;
        
        Ok(Self {
            inner: JavaObject::from_env(&mut env, entry_obj)?,
        })
    }
    
    pub(crate) fn inner(&self) -> &JavaObject {
        &self.inner
    }
}

/// Represents a suggestion for command completion
pub struct Suggestion {
    inner: JavaObject,
}

impl Suggestion {
    pub(crate) fn new(inner: JavaObject) -> Self {
        Self { inner }
    }
    
    /// Sets the start position for the suggestion
    pub fn set_start(&self, start: i32) -> Result<()> {
        let mut env = get_env()?;
        env.call_method(
            self.inner.as_obj()?,
            "setStart",
            "(I)V",
            &[JValue::Int(start)],
        )?;
        Ok(())
    }
    
    /// Sets the length for the suggestion
    pub fn set_length(&self, length: i32) -> Result<()> {
        let mut env = get_env()?;
        env.call_method(
            self.inner.as_obj()?,
            "setLength",
            "(I)V",
            &[JValue::Int(length)],
        )?;
        Ok(())
    }
    
    /// Adds a suggestion entry
    pub fn add_entry(&self, entry: &SuggestionEntry) -> Result<()> {
        let mut env = get_env()?;
        let entry_obj = entry.inner.as_obj()?;
        env.call_method(
            self.inner.as_obj()?,
            "addEntry",
            "(Lnet/minestom/server/command/builder/suggestion/SuggestionEntry;)V",
            &[JValue::Object(&entry_obj)],
        )?;
        Ok(())
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

        self.inner
            .call_bool_method("hasPermission", "(Ljava/lang/String;)Z", &[j_permission])
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
    pub fn register<T: Command + Send + Sync + 'static>(
        &self,
        command: T,
    ) -> Result<CommandBuilder> {
        let command = Arc::new(command);
        let command_name = command.name();
        let command_aliases = command.aliases();

        let mut env = get_env()?;

        // Create the command executor
        let callback_id = NEXT_CALLBACK_ID.fetch_add(1, Ordering::SeqCst);
        let command_clone = command.clone();
        let callback = Arc::new(move |sender: &CommandSender, context: &CommandContext| {
            command_clone.execute(sender, context)
        });
        COMMAND_CALLBACKS.write().insert(callback_id, callback);

        // Create the command executor
        let callback_class = env.find_class("rust/minestom/CommandExecutorCallback")?;
        let callback_obj =
            env.new_object(callback_class, "(J)V", &[JValue::Long(callback_id as i64)])?;

        // Create the command with aliases
        let command_class = env.find_class("net/minestom/server/command/builder/Command")?;
        let j_name = env.new_string(command_name)?;

        // Create aliases array
        let string_class = env.find_class("java/lang/String")?;
        let aliases_array =
            env.new_object_array(command_aliases.len() as i32, &string_class, JObject::null())?;
        for (i, alias) in command_aliases.iter().enumerate() {
            let j_alias = env.new_string(alias)?;
            env.set_object_array_element(&aliases_array, i as i32, &j_alias)?;
        }

        let command_obj = env.new_object(
            command_class,
            "(Ljava/lang/String;[Ljava/lang/String;)V",
            &[JValue::Object(&j_name), JValue::Object(&aliases_array)],
        )?;

        // Set the default executor
        env.call_method(
            &command_obj,
            "setDefaultExecutor",
            "(Lnet/minestom/server/command/builder/CommandExecutor;)V",
            &[JValue::Object(&callback_obj)],
        )?;

        // Create a global reference for the command object
        let global_command = env.new_global_ref(&command_obj)?;

        // Register the command
        self.inner.call_void_method(
            "register",
            "(Lnet/minestom/server/command/builder/Command;)V",
            &[JniValue::Object(command_obj)],
        )?;

        Ok(CommandBuilder::new(JavaObject::new(global_command)))
    }

    /// Unregisters a command by name
    pub fn unregister(&self, name: &str) -> Result<()> {
        let mut env = get_env()?;
        let j_name = name.to_java(&mut env)?;

        self.inner
            .call_void_method("unregister", "(Ljava/lang/String;)V", &[j_name])
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

    /// Adds a syntax with a required string argument to the command
    pub fn add_string_arg(&self, name: &str) -> Result<&Self> {
        let mut env = get_env()?;
        let j_name = name.to_java(&mut env)?;

        // Create ArgumentString
        let arg_class =
            env.find_class("net/minestom/server/command/builder/arguments/ArgumentString")?;
        let arg_obj = env.new_object(arg_class, "(Ljava/lang/String;)V", &[j_name.as_jvalue()])?;

        // Add syntax with the argument (using varargs)
        self.inner.call_object_method(
            "addSyntax",
            "(Lnet/minestom/server/command/builder/CommandExecutor;[Lnet/minestom/server/command/builder/arguments/Argument;)Ljava/util/Collection;",
            &[JniValue::Object(JObject::null()), JniValue::Object(arg_obj)],
        )?;

        Ok(self)
    }

    /// Adds a syntax with a required integer argument to the command
    pub fn add_integer_arg(&self, name: &str) -> Result<&Self> {
        let mut env = get_env()?;
        let j_name = name.to_java(&mut env)?;

        // Create ArgumentInteger
        let arg_class =
            env.find_class("net/minestom/server/command/builder/arguments/ArgumentInteger")?;
        let arg_obj = env.new_object(arg_class, "(Ljava/lang/String;)V", &[j_name.as_jvalue()])?;

        // Add syntax with the argument (using varargs)
        self.inner.call_object_method(
            "addSyntax",
            "(Lnet/minestom/server/command/builder/CommandExecutor;[Lnet/minestom/server/command/builder/arguments/Argument;)Ljava/util/Collection;",
            &[JniValue::Object(JObject::null()), JniValue::Object(arg_obj)],
        )?;

        Ok(self)
    }

    /// Adds a syntax with a required player argument to the command
    pub fn add_player_arg(&self, name: &str, only_players: bool) -> Result<&Self> {
        let mut env = get_env()?;
        let j_name = name.to_java(&mut env)?;

        // Create ArgumentEntity for players
        let arg_class = env
            .find_class("net/minestom/server/command/builder/arguments/minecraft/ArgumentEntity")?;
        let arg_obj = env.new_object(arg_class, "(Ljava/lang/String;)V", &[j_name.as_jvalue()])?;

        env.call_method(
            &arg_obj,
            "onlyPlayers",
            "(Z)Lnet/minestom/server/command/builder/arguments/minecraft/ArgumentEntity;",
            &[JniValue::Bool(only_players).as_jvalue()],
        )?;

        // Add syntax with the argument (using varargs)
        self.inner.call_object_method(
            "addSyntax",
            "(Lnet/minestom/server/command/builder/CommandExecutor;[Lnet/minestom/server/command/builder/arguments/Argument;)Ljava/util/Collection;",
            &[JniValue::Object(JObject::null()), JniValue::Object(arg_obj)],
        )?;

        Ok(self)
    }

    /// Adds a syntax with a greedy string argument (consumes all remaining text)
    pub fn add_greedy_string_arg(&self, name: &str) -> Result<&Self> {
        let mut env = get_env()?;
        let j_name = name.to_java(&mut env)?;

        // Create ArgumentStringArray with GREEDY mode
        let arg_class =
            env.find_class("net/minestom/server/command/builder/arguments/ArgumentStringArray")?;
        let arg_obj = env.new_object(arg_class, "(Ljava/lang/String;)V", &[j_name.as_jvalue()])?;

        // Add syntax with the argument (using varargs)
        self.inner.call_object_method(
            "addSyntax",
            "(Lnet/minestom/server/command/builder/CommandExecutor;[Lnet/minestom/server/command/builder/arguments/Argument;)Ljava/util/Collection;",
            &[JniValue::Object(JObject::null()), JniValue::Object(arg_obj)],
        )?;

        Ok(self)
    }

    /// Adds a syntax with multiple arguments to the command
    pub fn add_syntax_with_args(&self, args: &[&Argument]) -> Result<&Self> {
        let mut env = get_env()?;

        // Create argument array
        let arg_class = env.find_class("net/minestom/server/command/builder/arguments/Argument")?;
        let args_array = env.new_object_array(args.len() as i32, &arg_class, JObject::null())?;

        for (i, arg) in args.iter().enumerate() {
            let arg_obj = arg.inner.as_obj()?;
            env.set_object_array_element(&args_array, i as i32, &arg_obj)?;
        }

        // Add syntax with the arguments (array is passed as varargs)
        self.inner.call_object_method(
            "addSyntax",
            "(Lnet/minestom/server/command/builder/CommandExecutor;[Lnet/minestom/server/command/builder/arguments/Argument;)Ljava/util/Collection;",
            &[JniValue::Object(JObject::null()), JniValue::Object(JObject::from(args_array))],
        )?;

        Ok(self)
    }

    /// Sets a condition that must be met for the command to execute
    pub fn set_condition<F>(&self, condition: F) -> Result<()>
    where
        F: Fn(&CommandSender) -> Result<bool> + Send + Sync + 'static,
    {
        let mut env = get_env()?;

        // Store the condition callback
        let callback_id = NEXT_CALLBACK_ID.fetch_add(1, Ordering::SeqCst);
        let condition = Arc::new(condition);
        CONDITION_CALLBACKS.write().insert(callback_id, condition);

        // Create the condition executor
        let condition_class = env.find_class("rust/minestom/CommandConditionCallback")?;
        let condition_obj =
            env.new_object(condition_class, "(J)V", &[JValue::Long(callback_id as i64)])?;

        // Set the condition
        self.inner.call_void_method(
            "setCondition",
            "(Lnet/minestom/server/command/builder/condition/CommandCondition;)V",
            &[JniValue::Object(condition_obj)],
        )?;

        Ok(())
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_rust_minestom_SuggestionCallback_applySuggestion(
    env: *mut jni::sys::JNIEnv,
    _class: jni::objects::JClass,
    callback_id: jni::sys::jlong,
    sender: jni::objects::JObject,
    context: jni::objects::JObject,
    suggestion: jni::objects::JObject,
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
        let env = env_wrapper;

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

        let global_suggestion = match env.new_global_ref(&suggestion) {
            Ok(global) => global,
            Err(e) => {
                error!("Failed to create global reference for suggestion: {}", e);
                return;
            }
        };

        // Create JavaObjects from the global references
        let context_obj = JavaObject::new(global_context);
        let sender_obj = JavaObject::new(global_sender);
        let suggestion_obj = JavaObject::new(global_suggestion);

        // Create the command context, sender, and suggestion
        let context = CommandContext::new(context_obj);
        let sender = CommandSender::new(sender_obj);
        let mut suggestion = Suggestion::new(suggestion_obj);

        // Get the callback from our global map
        let callback = {
            let callbacks = SUGGESTION_CALLBACKS.read();
            match callbacks.get(&(callback_id as u64)) {
                Some(callback) => callback.clone(),
                None => {
                    error!("No suggestion callback found for id: {}", callback_id);
                    return;
                }
            }
        };

        debug!("Executing suggestion callback...");

        // Execute the callback
        if let Err(e) = callback(&sender, &context, &mut suggestion) {
            error!("Error executing suggestion callback: {}", e);
        }
    }));

    if let Err(e) = result {
        error!("Panic occurred in suggestion callback: {:?}", e);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_rust_minestom_CommandExecutorCallback_executeCommand(
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
        let env = env_wrapper;

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

#[unsafe(no_mangle)]
unsafe extern "system" fn Java_rust_minestom_CommandConditionCallback_checkCondition(
    env: *mut jni::sys::JNIEnv,
    _class: jni::objects::JClass,
    callback_id: jni::sys::jlong,
    sender: jni::objects::JObject,
) -> jni::sys::jboolean {
    unsafe {
        // Catch any panic to prevent unwinding into Java
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // Convert the raw JNIEnv pointer to a safe JNIEnv wrapper
            let env_wrapper = match jni::JNIEnv::from_raw(env) {
                Ok(env) => env,
                Err(e) => {
                    error!("Failed to get JNIEnv: {}", e);
                    return 0;
                }
            };

            // Create a mutable reference to the environment
            let env = env_wrapper;

            // Create a frame to automatically delete local references when we're done
            let _frame = match env.push_local_frame(64) {
                Ok(frame) => frame,
                Err(e) => {
                    error!("Failed to create local frame: {}", e);
                    return 0;
                }
            };

            // Create global reference to ensure the object stays alive
            let global_sender = match env.new_global_ref(&sender) {
                Ok(global) => global,
                Err(e) => {
                    error!("Failed to create global reference for sender: {}", e);
                    return 0;
                }
            };

            // Create JavaObject from the global reference
            let sender_obj = JavaObject::new(global_sender);

            // Create the command sender
            let sender = CommandSender::new(sender_obj);

            // Get the callback from our global map
            let callback = {
                let callbacks = CONDITION_CALLBACKS.read();
                match callbacks.get(&(callback_id as u64)) {
                    Some(callback) => callback.clone(),
                    None => {
                        error!("No callback found for id: {}", callback_id);
                        return 0;
                    }
                }
            };

            debug!("Executing command condition callback...");

            // Execute the callback
            match callback(&sender) {
                Ok(true) => 1,
                Ok(false) => 0,
                Err(e) => {
                    error!("Error in command condition: {}", e);
                    0
                }
            }
        }));

        match result {
            Ok(result) => result,
            Err(e) => {
                error!("Panic occurred in command condition callback: {:?}", e);
                0
            }
        }
    }
}
