use crate::command::CommandManager;
use crate::error::MinestomError;
use crate::event::EventHandler;
use crate::instance::InstanceManager;
use crate::jni_utils::{attach_jvm, get_env, JavaObject, JniValue, ToJava};
use crate::Result;
use jni::sys::JNIEnv;
use log::{debug, info, warn};
use std::path::Path;

#[derive(Clone)]
pub struct MinestomServer {
    inner: JavaObject,
}

impl MinestomServer {
    pub fn new() -> Result<Self> {
        let mut env = get_env()?;

        // Create the MinecraftServer instance
        let server = env.call_static_method(
            "net/minestom/server/MinecraftServer",
            "init",
            "()Lnet/minestom/server/MinecraftServer;",
            &[],
        )?;

        let server = server.l()?;
        let server = JavaObject::from_env(&mut env, server)?;

        Ok(Self { inner: server })
    }

    pub fn command_manager(&self) -> Result<CommandManager> {
        let result = self.inner.call_object_method(
            "getCommandManager",
            "()Lnet/minestom/server/command/CommandManager;",
            &[],
        )?;
        Ok(CommandManager::new(result))
    }

    pub fn start(&self, address: &str, port: u16) -> Result<()> {
        let mut env = get_env()?;

        // Convert the address to a Java string
        let address_jstring = env.new_string(address)?;

        // Call the start method
        self.inner.call_void_method(
            "start",
            "(Ljava/lang/String;I)V",
            &[
                JniValue::Object(address_jstring.into()),
                JniValue::Int(port as i32),
            ],
        )?;

        Ok(())
    }

    pub fn instance_manager(&self) -> Result<InstanceManager> {
        let mut env = get_env()?;

        let result = env.call_static_method(
            "net/minestom/server/MinecraftServer",
            "getInstanceManager",
            "()Lnet/minestom/server/instance/InstanceManager;",
            &[],
        )?;

        let java_obj = JavaObject::from_env(&mut env, result.l()?)?;
        Ok(InstanceManager::new(java_obj))
    }

    pub fn event_handler(&self) -> Result<EventHandler> {
        let mut env = get_env()?;

        let result = env.call_static_method(
            "net/minestom/server/MinecraftServer",
            "getGlobalEventHandler",
            "()Lnet/minestom/server/event/GlobalEventHandler;",
            &[],
        )?;

        let java_obj = JavaObject::from_env(&mut env, result.l()?)?;
        Ok(EventHandler::new(java_obj))
    }
}
