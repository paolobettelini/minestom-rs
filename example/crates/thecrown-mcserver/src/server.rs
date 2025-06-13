use crate::advancements::init_player_advancements;
use crate::logic::lobby::LobbyServer;
use crate::logic::parkour::ParkourServer;
use crate::maps::LobbyMap2;
use crate::maps::map::LobbyMap;
use crate::mojang::get_skin_and_signature;
use thecrown_protocol::GameServerSpecs;
use thecrown_protocol::GameServerType;
use log::info;
use minestom;
use thecrown_protocol::RelayPacket;
use thecrown_common::nats::CallbackType;
use minestom::MinestomServer;
use thecrown_protocol::McServerPacket;
use minestom::ServerListPingEvent;
use minestom::TOKIO_HANDLE;
use minestom::entity::PlayerSkin;
use minestom::{
    component,
    event::player::{AsyncPlayerConfigurationEvent, PlayerSkinInitEvent, PlayerSpawnEvent},
    material::Material,
    resource_pack::{ResourcePackInfo, ResourcePackRequestBuilder},
};
use minestom::instance::InstanceManager;
use rand::Rng;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::{Mutex, RwLock};
use thecrown_common::nats::NatsClient;
use uuid::Uuid;
use world_seed_entity_engine::model_engine::ModelEngine;

static PLAYER_SERVER: LazyLock<RwLock<HashMap<Uuid, String>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

static SERVERS: LazyLock<Mutex<HashMap<String, Arc<Box<dyn Server + Send + Sync>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub trait Server: Send + Sync {
    fn init(&self, minecraft_server: &MinestomServer) -> minestom::Result<()>;
    fn init_player(
        &self,
        minecraft_server: &MinestomServer,
        config_event: &AsyncPlayerConfigurationEvent,
    ) -> minestom::Result<()>;
}

type PacketType = McServerPacket;
#[derive(Clone)]
pub struct State {
    minecraft_server: MinestomServer,
    instance_manager: InstanceManager,
}

pub async fn handle_msg(state: &State, msg: PacketType) -> Option<PacketType> {
    match msg { // TODO voglio ricever un Arc<State>
        PacketType::StartGameServers { servers } => {
            for server_specs in servers {
                log::info!("Starting server {:?}", server_specs);
                let server: Box<dyn Server + Send + Sync> = match server_specs.server_type {
                    GameServerType::Lobby => {
                        let map = LobbyMap2::new(&state.instance_manager).expect("Could not create map");
                        let server = LobbyServer::new(map, state.minecraft_server.clone()).expect("Could not create expect");
                        Box::new(server)                        
                    },
                    GameServerType::Parkour => {
                        let server = ParkourServer::default();
                        Box::new(server)
                    },
                };

                let _ = server.init(&state.minecraft_server);
                let server_name = String::from(server_specs.name);
                SERVERS
                    .lock()
                    .unwrap()
                    .insert(server_name.clone(), Arc::new(server));
            }
            None
        }
        _ => None,
    }
}

pub async fn run_server() -> anyhow::Result<()> {
    init_logging();
    
    let minecraft_server = MinestomServer::new()?;
    let scheduler = minecraft_server.scheduler_manager()?;
    let instance_manager = minecraft_server.instance_manager()?;
    let command_manager = minecraft_server.command_manager()?;

    let server_name = "server1";
    let nats_url = String::from("127.0.0.1:4222");
    let subject = format!("mcserver.{}", server_name);
    let nats_client = Arc::new(NatsClient::new(nats_url).await?);
    let state = State {
        minecraft_server: minecraft_server.clone(),
        instance_manager: instance_manager.clone(),
    };
    let async_handler: Arc<CallbackType<_, _>> =
        Arc::new(|state, msg| Box::pin(handle_msg(state, msg)));
    let nats_client_inner = nats_client.clone();
    let task_handle = tokio::task::spawn(async move {
        nats_client_inner
            .handle_subscription_with_subject(subject, state.clone(), async_handler.as_ref())
            .await;
    });
    // let out = task_handle.await?;
    let register_packet = RelayPacket::RegisterServer {
        server_name: server_name.to_string(),
        address: String::from("127.0.0.1"),
        port: 25565
    };

    // WorldEntitySeedEngine initialization
    let output_dir = "/home/paolo/Desktop/github/minestom-rs/example/resources/output";
    let output_dir = Path::new(output_dir);
    let models_dir = output_dir.join("models");
    let mappings = output_dir.join("model_mapping.json");
    ModelEngine::set_model_material(Material::MagmaCream)?;
    ModelEngine::load_mappings(mappings, models_dir)?;

    let minecraft_server_clone = minecraft_server.clone();
    let event_handler = minecraft_server.event_handler()?;

    event_handler.listen(move |config_event: &AsyncPlayerConfigurationEvent| {
        // Try to get player information
        if let Ok(player) = config_event.player() {
            if let Ok(name) = player.get_username() {
                info!("Player configured: {}", name);

                // FAKE: the player needs to be sent to lobbysrv1
                let servers = vec!["lobby1", "parkour1"];
                let server_name = servers[rand::thread_rng().gen_range(0..servers.len())];
                log::info!("Sending player {} to server: {}", name, server_name);
                SERVERS
                    .lock()
                    .unwrap()
                    .get(server_name)
                    .unwrap()
                    .init_player(&minecraft_server_clone, &config_event)?;
                PLAYER_SERVER
                    .write()
                    .unwrap()
                    .insert(player.get_uuid()?, server_name.to_string());

                // Send resource pack
                let uuid = uuid::Uuid::new_v4();
                let url = "http://127.0.0.1:6543/resourcepack.zip";
                let hash = include_str!(concat!(env!("OUT_DIR"), "/resourcepack.sha1"));

                let pack_info = ResourcePackInfo::new(uuid, url, hash)?;
                let request = ResourcePackRequestBuilder::new()?
                    .packs(pack_info)?
                    .prompt(&component!("Please accept the resource pack").gold())?
                    .required(true)?
                    .build()?;

                player.send_resource_packs(&request)?;
            }
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
                let (texture, signature) = TOKIO_HANDLE
                    .block_on(async { get_skin_and_signature(uuid).await })
                    .unwrap();

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

    let server = minecraft_server.clone();
    event_handler.listen(move |spawn_event: &PlayerSpawnEvent| {
        if let Ok(player) = spawn_event.player() {
            let _ = init_player_advancements(&server, &player);
        }
        Ok(())
    })?;

    info!("Starting server on 0.0.0.0:25565...");
    minecraft_server.start("0.0.0.0", 25565)?;

    // Register to relay
    nats_client.publish(&register_packet).await;

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
