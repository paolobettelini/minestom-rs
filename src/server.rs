use crate::Result;
use crate::command::CommandManager;
use crate::event::EventHandler;
use crate::instance::InstanceManager;
use crate::jni_utils::{JavaObject, get_env};

#[derive(Clone)]
pub struct MinestomServer {
    inner: JavaObject,
}

impl MinestomServer {
    /// Creates a new MinestomServer instance.
    /// This initializes the Minecraft server and returns a handle to it.
    pub fn new() -> Result<Self> {
        let mut env = get_env()?;
        let server_class = env.find_class("net/minestom/server/MinecraftServer")?;
        let server = env.call_static_method(
            &server_class,
            "init",
            "()Lnet/minestom/server/MinecraftServer;",
            &[],
        )?;
        let server_obj = server.l()?;
        let server_ref = env.new_global_ref(server_obj)?;
        Ok(Self {
            inner: JavaObject::new(server_ref),
        })
    }

    pub fn start(&self, address: &str, port: u16) -> Result<()> {
        let mut env = get_env()?;
        let j_address = env.new_string(address)?;
        self.inner.call_void_method(
            "start",
            "(Ljava/lang/String;I)V",
            &[j_address.into(), (port as i32).into()],
        )
    }

    pub fn instance_manager(&self) -> Result<InstanceManager> {
        let mut env = get_env()?;
        let server_class = env.find_class("net/minestom/server/MinecraftServer")?;
        let instance_manager = env.call_static_method(
            &server_class,
            "getInstanceManager",
            "()Lnet/minestom/server/instance/InstanceManager;",
            &[],
        )?;
        let instance_manager_obj = instance_manager.l()?;
        let instance_manager_global = env.new_global_ref(instance_manager_obj)?;
        Ok(InstanceManager::new(JavaObject::new(
            instance_manager_global,
        )))
    }

    pub fn event_handler(&self) -> Result<EventHandler> {
        let mut env = get_env()?;
        let server_class = env.find_class("net/minestom/server/MinecraftServer")?;
        let event_handler = env.call_static_method(
            &server_class,
            "getGlobalEventHandler",
            "()Lnet/minestom/server/event/GlobalEventHandler;",
            &[],
        )?;
        let event_handler_obj = event_handler.l()?;
        let event_handler_global = env.new_global_ref(event_handler_obj)?;
        Ok(EventHandler::new(JavaObject::new(event_handler_global)))
    }

    /// Gets the command manager for registering and managing commands
    pub fn command_manager(&self) -> Result<CommandManager> {
        let mut env = get_env()?;
        let server_class = env.find_class("net/minestom/server/MinecraftServer")?;
        let command_manager = env.call_static_method(
            &server_class,
            "getCommandManager",
            "()Lnet/minestom/server/command/CommandManager;",
            &[],
        )?;
        let command_manager_obj = command_manager.l()?;
        let command_manager_global = env.new_global_ref(command_manager_obj)?;
        Ok(CommandManager::new(JavaObject::new(command_manager_global)))
    }
}
