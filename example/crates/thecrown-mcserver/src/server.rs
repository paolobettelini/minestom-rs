use log::info;
use minestom::{
    self, component, entity::PlayerSkin, event::player::{AsyncPlayerConfigurationEvent, PlayerSkinInitEvent, PlayerSpawnEvent}, instance::InstanceManager, material::Material, resource_pack::{ResourcePackInfo, ResourcePackRequestBuilder}, InstanceContainer, MinestomServer, Player, ServerListPingEvent, TOKIO_HANDLE
};
use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, LazyLock, Mutex, RwLock},
};
use thecrown_advancements::init_player_advancements;
use thecrown_commands::{WebloginCommand, WhisperCommand};
use thecrown_common::{
    map::create_lobby_instance_container,
    mojang::get_skin_and_signature,
    nats::{CallbackType, NatsClient},
    player::{COOKIE_AUTH, GetGameServer},
    server::Server,
};
use thecrown_lobby::{LobbyServer, commands::SpawnCommand, maps::LobbyMap2};
use thecrown_parkour::ParkourServer;
use thecrown_protocol::{GameServerType, McServerPacket, RelayPacket};
use uuid::Uuid;
use world_seed_entity_engine::model_engine::ModelEngine;

static SERVERS: LazyLock<Mutex<HashMap<String, Arc<Box<dyn Server + Send + Sync>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new())); // RwLock

// Username, player
static PLAYERS: LazyLock<Mutex<HashMap<String, Player>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

type PacketType = McServerPacket;
#[derive(Clone)]
pub struct State {
    minecraft_server: MinestomServer,
    instance_manager: InstanceManager,
    lobby_instance_container: InstanceContainer,
}

pub async fn handle_msg(state: &State, msg: PacketType) -> Option<PacketType> {
    match msg {
        // TODO voglio ricever un Arc<State>
        PacketType::StartGameServers { servers } => {
            for server_specs in servers {
                log::info!("Starting server {:?}", server_specs);
                let server: Box<dyn Server + Send + Sync> = match server_specs.server_type {
                    GameServerType::Lobby => {
                        let shared_instance = state
                            .instance_manager
                            .create_shared_instance(&state.lobby_instance_container)
                            .expect("Could not create shared instance");

                        log::info!("Created shared instance for lobby: {}", server_specs.name);

                        let map = LobbyMap2::new(shared_instance).expect("Could not create map");
                        let server = LobbyServer::new(map, state.minecraft_server.clone())
                            .expect("Could not create server");
                        Box::new(server)
                    }
                    GameServerType::Parkour => {
                        let server = ParkourServer::default();
                        Box::new(server)
                    }
                };

                let res = server.init(&state.minecraft_server);
                if let Err(e) = res {
                    log::error!("{:?}", e);
                }

                let server_name = String::from(server_specs.name);
                SERVERS
                    .lock()
                    .unwrap()
                    .insert(server_name.clone(), Arc::new(server));
            }
            None
        }
        PacketType::WhisperCommand { sender, target, message } => {
            let res = {
                let guard = PLAYERS.lock().unwrap();
                guard.get(&sender).cloned()
            };
            if let Some(player) = res {
                let msg = component!("Whisper >> ")
                    .gray()
                    .chain(component!("[").gray())
                    .chain(component!("{}", sender).yellow())
                    .chain(component!("] ").gray())
                    .chain(component!(" {}", message).white());
                let _ = player.send_message(&msg);
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
    let port = 25566;
    let nats_url = String::from("127.0.0.1:4222");
    let subject = format!("mcserver.{}", server_name);
    let nats_client = Arc::new(NatsClient::new(nats_url).await?);
    let lobby_instance_container = create_lobby_instance_container(&instance_manager)
        .expect("Could not create instance container");
    let state = State {
        minecraft_server: minecraft_server.clone(),
        instance_manager: instance_manager.clone(),
        lobby_instance_container: lobby_instance_container.clone(),
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
        port,
    };

    // WorldEntitySeedEngine initialization
    let output_dir = "/home/paolo/Desktop/github/minestom-rs/example/resources/output";
    let output_dir = Path::new(output_dir);
    let models_dir = output_dir.join("models");
    let mappings = output_dir.join("model_mapping.json");
    ModelEngine::set_model_material(Material::MagmaCream)?;
    ModelEngine::load_mappings(mappings, models_dir)?;

    // Register commands
    SpawnCommand.register(&command_manager)?;
    WebloginCommand.register(&command_manager)?;
    WhisperCommand::new(nats_client.clone()).register(&command_manager)?;

    let minecraft_server_clone = minecraft_server.clone();
    let event_handler = minecraft_server.event_handler()?;

    let nats = nats_client.clone();
    event_handler.listen(move |config_event: &AsyncPlayerConfigurationEvent| {
        // Try to get player information
        if let Ok(player) = config_event.player() {
            if let Ok(username) = player.get_username() {
                info!("Configuring player: {}", username);

                if let Some(cookie) = player.get_player_connection()?.fetch_cookie(COOKIE_AUTH)? {
                    // Send AuthUserJoin to Relay
                    let packet = RelayPacket::AuthUserJoin {
                        username: username.clone(),
                        server: server_name.to_string(),
                        cookie,
                    };
                    let response = TOKIO_HANDLE.block_on(async { nats.request(&packet).await });

                    if let Some(RelayPacket::ServeAuthResult { game_server }) = response {
                        if let Some(game_server) = game_server {
                            log::info!(
                                "Sending player {} to game server: {}",
                                username,
                                game_server
                            );
                            let server = {
                                let servers_guard = SERVERS.lock().unwrap();
                                servers_guard.get(&game_server).unwrap().clone()
                            };
                            server.init_player(&minecraft_server_clone, &config_event)?;
                            // Something like if there server isn't there check another list of
                            // servers that are being created and wait.
                            let res = player.set_server(server.clone());
                            if let Err(e) = res {
                                log::error!("Error setting setver: {:?}", e);
                            }
                            // Add player to global player list
                            {
                                let mut guard = PLAYERS.lock().unwrap();
                                guard.insert(username, player.clone());
                            }

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
                        } else {
                            log::warn!("Player {} bad auth token", username);
                            player.kick(&component!("Bad authentication token").red())?;
                        }
                    } else {
                        log::warn!("Player {} bad game server", username);
                        player.kick(&component!("This game server does not exist").red())?;
                    }
                } else {
                    log::warn!("Player {} no cookie", username);
                    player.kick(&component!("No cookie provided").red())?;
                }
            }
        }

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

    event_handler.listen(move |event: &ServerListPingEvent| {
        event.set_cancelled(true)?;
        Ok(())
    })?;

    minecraft_server.start("0.0.0.0", port)?;

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
