use crate::advancement::AdvancementManager;
use crate::Result;
use crate::command::CommandManager;
use crate::entity::Player;
use crate::event::EventNode;
use crate::instance::InstanceManager;
use crate::jni_utils::{JavaObject, get_env};
use crate::scheduler::SchedulerManager;
use jni::objects::JValue;
use uuid::Uuid;

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

    /// Gets a player by their UUID
    pub fn get_player_by_uuid(&self, uuid: Uuid) -> Result<Option<Player>> {
        let mut env = get_env()?;

        // Convert Rust UUID to Java UUID
        let uuid_class = env.find_class("java/util/UUID")?;
        let uuid_str = uuid.to_string();
        let uuid_jstring = env.new_string(&uuid_str)?;
        let uuid_obj = env.call_static_method(
            uuid_class,
            "fromString",
            "(Ljava/lang/String;)Ljava/util/UUID;",
            &[JValue::Object(&uuid_jstring)],
        )?;

        // Get the connection manager
        let server_class = env.find_class("net/minestom/server/MinecraftServer")?;
        let connection_manager = env.call_static_method(
            server_class,
            "getConnectionManager",
            "()Lnet/minestom/server/network/ConnectionManager;",
            &[],
        )?;

        let connection_manager_obj = connection_manager.l()?;
        let uuid_obj = uuid_obj.l()?;

        // Get the player
        let player = env.call_method(
            &connection_manager_obj,
            "getPlayer",
            "(Ljava/util/UUID;)Lnet/minestom/server/entity/Player;",
            &[JValue::Object(&uuid_obj)],
        )?;

        let player_obj = player.l()?;
        if player_obj.is_null() {
            Ok(None)
        } else {
            Ok(Some(Player::new(JavaObject::new(
                env.new_global_ref(player_obj)?,
            ))))
        }
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

    pub fn event_handler(&self) -> Result<EventNode> {
        let mut env = get_env()?;

        // Get the MinecraftServer class
        let server_class = env.find_class("net/minestom/server/MinecraftServer")?;

        // Call the static method
        let event_handler = env.call_static_method(
            server_class,
            "getGlobalEventHandler",
            "()Lnet/minestom/server/event/GlobalEventHandler;",
            &[],
        )?;

        // Convert to JavaObject and create EventNode
        let event_handler_obj = event_handler.l()?;
        let event_handler_global = env.new_global_ref(event_handler_obj)?;
        Ok(EventNode::from(JavaObject::new(event_handler_global)))
    }

    /// Gets the advancement manager for creating custom advancement tabs
    pub fn advancement_manager(&self) -> Result<AdvancementManager> {
        let mut env = get_env()?;
        // Trova la classe MinecraftServer
        let server_class = env.find_class("net/minestom/server/MinecraftServer")?;
        // Chiama il metodo statico getAdvancementManager(): AdvancementManager
        let manager_value = env.call_static_method(
            &server_class,
            "getAdvancementManager",
            "()Lnet/minestom/server/advancements/AdvancementManager;",
            &[],
        )?;
        // Estrai l'oggetto Java
        let manager_obj = manager_value.l()?;
        // Converti in global ref per Rust
        let manager_ref = env.new_global_ref(manager_obj)?;
        // Avvolgi nel tuo struct Rust
        Ok(AdvancementManager { inner: JavaObject::new(manager_ref) })
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

    /// Gets the scheduler manager for scheduling tasks
    pub fn scheduler_manager(&self) -> Result<SchedulerManager> {
        let mut env = get_env()?;
        let server_class = env.find_class("net/minestom/server/MinecraftServer")?;
        let scheduler_manager = env.call_static_method(
            &server_class,
            "getSchedulerManager",
            "()Lnet/minestom/server/timer/SchedulerManager;",
            &[],
        )?;
        let scheduler_manager_obj = scheduler_manager.l()?;
        let scheduler_manager_global = env.new_global_ref(scheduler_manager_obj)?;
        Ok(SchedulerManager::new(JavaObject::new(
            scheduler_manager_global,
        )))
    }
}
