use crate::jni_utils::{get_env, JavaObject, JniValue, ToJava};
use crate::Result;
use crate::text::Component;
use jni::objects::{JObject, JString};
use jni::sys::jobject;

/// Represents a command that can be registered with the command manager
pub trait Command {
    /// Returns the name of the command
    fn name(&self) -> &str;
    
    /// Returns a list of aliases for the command
    fn aliases(&self) -> Vec<&str> {
        Vec::new()
    }
    
    /// Called when the command is executed
    fn execute(&self, sender: &CommandSender, args: &[String]) -> Result<()>;
}

/// Represents an entity that can execute commands
pub struct CommandSender {
    inner: JavaObject,
}

impl CommandSender {
    pub(crate) fn new(inner: JavaObject) -> Self {
        Self { inner }
    }

    /// Sends a message to the command sender
    pub fn send_message(&self, message: &Component) -> Result<()> {
        let mut env = get_env()?;
        let j_message = message.to_java(&mut env)?;
        
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
        let command_obj = env.new_object(
            command_class,
            "(Ljava/lang/String;)V",
            &[j_name.as_jvalue()],
        )?;
        
        // Register aliases if any
        let aliases = command.aliases();
        if !aliases.is_empty() {
            let j_aliases: Vec<JString> = aliases
                .iter()
                .map(|alias| env.new_string(alias))
                .collect::<jni::errors::Result<_>>()?;
                
            let j_array = env.new_object_array(
                j_aliases.len() as i32,
                "java/lang/String",
                JObject::null(),
            )?;
            
            for (i, alias) in j_aliases.iter().enumerate() {
                env.set_object_array_element(&j_array, i as i32, alias)?;
            }
            
            let array_obj = env.new_local_ref(&j_array)?;
            env.call_method(
                &command_obj,
                "setAliases",
                "([Ljava/lang/String;)Lnet/minestom/server/command/builder/Command;",
                &[JniValue::Object(array_obj).as_jvalue()],
            )?;
        }
        
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
            "addStringArg",
            "(Ljava/lang/String;)Lnet/minestom/server/command/builder/Command;",
            &[j_name],
        )?;
        
        Ok(self)
    }

    /// Adds a required integer argument to the command
    pub fn add_integer_arg(&self, name: &str) -> Result<&Self> {
        let mut env = get_env()?;
        let j_name = name.to_java(&mut env)?;
        
        self.inner.call_void_method(
            "addIntegerArg",
            "(Ljava/lang/String;)Lnet/minestom/server/command/builder/Command;",
            &[j_name],
        )?;
        
        Ok(self)
    }

    /// Sets a condition that must be met for the command to execute
    pub fn set_condition<F>(&self, condition: F) -> Result<&Self>
    where
        F: Fn(&CommandSender) -> bool + 'static,
    {
        // TODO: Implement condition setting via JNI
        Ok(self)
    }
}
