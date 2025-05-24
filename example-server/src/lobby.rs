use crate::commands::SpawnCommand;
use crate::magic_values::*;
use crate::maps::LobbyMap2;
use crate::maps::map::LobbyMap;
use crate::mojang::get_skin_and_signature;
use log::{info, error};
use minestom::MinestomServer;
use minestom_rs as minestom;
use minestom_rs::ServerListPingEvent;
use minestom_rs::TOKIO_HANDLE;
use minestom_rs::InstanceContainer;
use minestom_rs::entity::PlayerSkin;
use minestom_rs::{
    attribute::Attribute,
    command::{Command, CommandContext},
    component,
    entity::GameMode,
    event::player::{
        AsyncPlayerConfigurationEvent, PlayerMoveEvent, PlayerSkinInitEvent, PlayerSpawnEvent,
    },
    item::{InventoryHolder, ItemStack},
    material::Material,
    resource_pack::{ResourcePackInfo, ResourcePackRequest, ResourcePackRequestBuilder},
};
use std::sync::LazyLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::{Mutex, RwLock};
use uuid::Uuid;
use parking_lot::Mutex as ParkingMutex;

static PLAYER_SERVER: LazyLock<RwLock<HashMap<Uuid, String>>> = 
    LazyLock::new(|| RwLock::new(HashMap::new()));

static SERVERS: LazyLock<Mutex<HashMap<String, Arc<Box<dyn Server + Send + Sync>>>>> = 
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub trait Server: Send + Sync {
    fn init(&self, minecraft_server: &MinestomServer) -> minestom::Result<()>;
    fn init_player(&self, config_event: &AsyncPlayerConfigurationEvent) -> minestom::Result<()>;
}

pub struct LobbyServer<T: LobbyMap> {
    map: T,
}

impl<T: LobbyMap> LobbyServer<T> {
    pub fn new(map: T) -> minestom::Result<Self> {
        Ok(LobbyServer { map })
    }
}

impl<T: LobbyMap> Server for LobbyServer<T> {
    fn init_player(&self, config_event: &AsyncPlayerConfigurationEvent) -> minestom::Result<()> {
        if let Ok(player) = config_event.player() {
            info!("Setting spawning instance for player");
            config_event.spawn_instance(&self.map.instance())?;
        }

        Ok(())
    }

    fn init(&self, minecraft_server: &MinestomServer) -> minestom::Result<()> {
        let scheduler = minecraft_server.scheduler_manager()?;
        let instance_manager = minecraft_server.instance_manager()?;

        // TODO use ShareInstance from a static instance
        let instance = self.map.instance();

        let event_handler = instance.event_node()?;
        let spawn_instance = instance.clone();

        let map_clone = self.map.clone();
        event_handler.listen(move |spawn_event: &PlayerSpawnEvent| {
            info!("Player spawn event triggered");
            if let Ok(player) = spawn_event.player() {
                let username = player.get_username()?;

                let welcome_msg = component!("Welcome to the server, {}!", username)
                    .gold()
                    .bold();
                let info_msg = component!("Enjoy your adventure!").green().italic();
                let message = welcome_msg.chain_newline(info_msg);

                player.send_message(&message)?;
                player.set_game_mode(GameMode::Adventure)?;

                let (x, y, z, yaw, pitch) = map_clone.spawn_coordinate();
                player.teleport(x, y, z, yaw, pitch)?;
                player.set_allow_flying(true)?;

                // https://minecraft.wiki/w/Attribute#Modifiers
                let scale = distribution(AVG_SCALE, MIN_SCALE, MAX_SCALE);
                //let scale = 15.0;
                info!("Setting player scale to {}", scale);
                player
                    .get_attribute(Attribute::Scale)?
                    .set_base_value(scale)?;
                player
                    .get_attribute(Attribute::JumpStrength)?
                    .set_base_value(jump_strength_scale(scale))?;
                player
                    .get_attribute(Attribute::StepHeight)?
                    .set_base_value(step_height_scale(scale))?;

                // Create a diamond sombrero
                let sombrero = ItemStack::of(Material::Diamond)?
                    .with_amount(1)?
                    .with_custom_model_data("piano")?;

                // Get player's inventory and set the helmet
                let inventory = player.get_inventory()?;
                inventory.set_helmet(&sombrero)?;
            }
            Ok(())
        })?;

        Ok(())
    }
}

pub async fn run_server() -> minestom::Result<()> {
    init_logging();

    let minecraft_server = MinestomServer::new()?;
    let scheduler = minecraft_server.scheduler_manager()?;
    let instance_manager = minecraft_server.instance_manager()?;
    let command_manager = minecraft_server.command_manager()?;

    // FAKE: create a server
    let map = LobbyMap2::new(&instance_manager)?;
    let server = LobbyServer::new(map)?;
    server.init(&minecraft_server)?;
    let server = Box::new(server);
    let server_name = String::from("lobbysrv1");
    SERVERS.lock().unwrap().insert(server_name.clone(), Arc::new(server));

    // Register commands
    // command_manager.register(SpawnCommand::new(map.clone()))?;

    let event_handler = minecraft_server.event_handler()?;

    event_handler.listen(move |config_event: &AsyncPlayerConfigurationEvent| {
        // Try to get player information
        if let Ok(player) = config_event.player() {
            if let Ok(name) = player.get_username() {
                info!("Player configured: {}", name);
            }

            // FAKE: the player needs to be sent to lobbysrv1
            SERVERS.lock().unwrap().get(&"lobbysrv1".to_string())
                .unwrap()
                .init_player(&config_event)?;
            PLAYER_SERVER.write().unwrap().insert(player.get_uuid()?, "lobbysrv1".to_string());

            // Send resource pack
            let uuid = uuid::Uuid::new_v4();
            let url = "http://127.0.0.1:8080/resourcepack.zip";
            let hash = "123456";

            let pack_info = ResourcePackInfo::new(uuid, url, hash)?;
            let request = ResourcePackRequestBuilder::new()?
                .packs(pack_info)?
                .prompt(&component!("Please accept the resource pack").gold())?
                .required(true)?
                .build()?;

            player.send_resource_packs(&request)?;
        }

        Ok(())
    })?;

    // TODO: move to auth server
    event_handler.listen(move |event: &ServerListPingEvent| {
        let response_data = event.get_response_data()?;

        response_data.set_online(-1)?;
        response_data.set_max_player(i32::MAX)?;
        response_data.set_description(&component!("Henlo").red())?;
        response_data.set_favicon(&crate::favicon::random_image())?;

        Ok(())
    })?;

    // Does not work
    /*scheduler
        .build_task(move || {
            if let Err(err) = (|| -> minestom::Result<()> {
                TOKIO_HANDLE.block_on(async {
                    println!("Test task executing!");
                    Ok(())
                })
            })() {
                error!("Task error: {}", err);
            }
            Ok(())
        })?
        .repeat(1)?
        .schedule()?;*/

    event_handler.listen(move |skin_event: &PlayerSkinInitEvent| {
        info!("Player skin init event triggered");
        if let Ok(player) = skin_event.player() {
            if let Ok(uuid) = player.get_uuid() {
                let (texture, signature) = TOKIO_HANDLE.block_on(async {
                    get_skin_and_signature(uuid).await
                }).unwrap();

                let skin = PlayerSkin::create(&texture, &signature)?;
                skin_event.set_skin(&skin)?;
            }
        }
        Ok(())
    })?;

    /*let scheduler = scheduler.clone();
    event_handler.listen_async(move |skin_event: PlayerSkinInitEvent| {
        let scheduler = scheduler.clone();
        async move {
            info!("Player skin init event triggered");
            if let Ok(player) = skin_event.player() {
                if let Ok(uuid) = player.get_uuid() {
                    info!("Got player UUID: {}", uuid);
                    
                    let (texture, signature) = get_skin_and_signature(uuid).await.unwrap();
                    let skin = PlayerSkin::create(&texture, &signature)?;
                }
            }
            Ok(())
        }
    })?;*/

    info!("Starting server on 0.0.0.0:25565...");
    minecraft_server.start("0.0.0.0", 25565)?;

    info!("Server is now listening for connections!");
    info!("Press Ctrl+C to stop the server");

    // Keep the main thread alive
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C");

    info!("Shutting down server...");

    Ok(())
}

fn init_logging() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp(Some(env_logger::fmt::TimestampPrecision::Millis))
        .init();
}
