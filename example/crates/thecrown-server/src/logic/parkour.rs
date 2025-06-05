use crate::server::Server;
use log::error;
use log::info;
use minestom;
use minestom::event::EventNode;
use minestom::{BlockType, MinestomServer, Position};
use minestom::{
    component,
    entity::{GameMode, Player},
    event::{
        Event,
        player::{
            AsyncPlayerConfigurationEvent, PlayerChatEvent, PlayerDisconnectEvent, PlayerMoveEvent,
            PlayerSpawnEvent,
        },
        server::ServerListPingEvent,
    },
    instance::InstanceContainer,
    sound::{Sound, SoundEvent, Source},
};
use parking_lot::RwLock;
use rand::Rng;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const START_POS: (i32, i32, i32) = (0, 100, 0);
const BLOCK_TYPES: &[BlockType] = &[
    BlockType::GrassBlock,
    BlockType::OakLog,
    BlockType::BirchLog,
    BlockType::OakLeaves,
    BlockType::BirchLeaves,
    BlockType::Dirt,
    BlockType::MossyCobblestone,
    BlockType::Netherrack,
    BlockType::Glowstone,
];

#[derive(Clone)]
struct Vec3 {
    x: i32,
    y: i32,
    z: i32,
}

impl Vec3 {
    fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

struct GameState {
    blocks: VecDeque<Vec3>,
    score: i32,
    combo: i32,
    target_y: i32,
    last_block_timestamp: u64,
    instance: InstanceContainer,
}

impl GameState {
    fn new(instance: InstanceContainer) -> Self {
        let state = Self {
            blocks: VecDeque::new(),
            score: 0,
            combo: 0,
            target_y: 0,
            last_block_timestamp: 0,
            instance,
        };
        state.instance.set_time_rate(0).unwrap();
        state
    }
}

pub struct ParkourServer {
    player_states: Arc<Mutex<HashMap<String, GameState>>>,
    player_uuids: Arc<RwLock<HashMap<Uuid, Player>>>,
}

impl Default for ParkourServer {
    fn default() -> Self {
        Self {
            player_states: Arc::new(Mutex::new(HashMap::new())),
            player_uuids: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Server for ParkourServer {
    fn init_player(
        &self,
        minecraft_server: &MinestomServer,
        config_event: &AsyncPlayerConfigurationEvent,
    ) -> minestom::Result<()> {
        if let Ok(player) = config_event.player() {
            if let Ok(name) = player.get_username() {
                info!("Creating empty instance and game state for player");
                let instance = create_empty_instance(&minecraft_server)?;
                let game_state = GameState::new(instance.clone());
                self.player_states
                    .lock()
                    .unwrap()
                    .insert(name.clone(), game_state);

                // Store player in the HashMap
                if let Ok(uuid) = player.get_uuid() {
                    self.player_uuids.write().insert(uuid, player);
                }

                config_event.spawn_instance(&instance)?;
            }
        }

        Ok(())
    }

    fn init(&self, minecraft_server: &MinestomServer) -> minestom::Result<()> {
        let states_ref = self.player_states.clone();
        let uuids_ref = self.player_uuids.clone();

        let event_node = EventNode::create_player_filter("parkour", move |player| {
            if let Ok(uuid) = player.get_uuid() {
                uuids_ref.read().contains_key(&uuid)
            } else {
                false
            }
        })?;

        let event_handler = minecraft_server.event_handler()?;
        event_handler.add_child(&event_node)?;

        let uuids_ref = self.player_uuids.clone();
        event_node.listen(move |event: &PlayerChatEvent| {
            event.set_cancelled(true)?;

            let player = event.player()?;
            let raw_msg = event.raw_message()?;
            let username = player.get_username()?;
            let formatted = component!("[{}] {}", username, raw_msg);

            // Send to all players
            let players = uuids_ref.read();
            for player in players.values() {
                player.send_message(&formatted)?;
            }

            Ok(())
        })?;

        event_node.listen(move |spawn_event: &PlayerSpawnEvent| {
            info!("Player spawn event triggered");
            if let Ok(player) = spawn_event.player() {
                let username = player.get_username()?;

                if let Some(state) = states_ref.lock().unwrap().get_mut(&username) {
                    player.set_game_mode(GameMode::Adventure)?;
                    reset_player(&player, state)?;
                }
            }
            Ok(())
        })?;

        let states_ref = self.player_states.clone();
        let uuids_ref = self.player_uuids.clone();
        event_node.listen(move |disconnect_event: &PlayerDisconnectEvent| {
            info!("Player disconnect event triggered");
            if let Ok(player) = disconnect_event.player() {
                if let Ok(username) = player.get_username() {
                    info!("Player disconnected, removing game state");
                    // Remove the player's game state
                    states_ref.lock().unwrap().remove(&username);
                    info!("Game state removed for {}", username);

                    // Remove from players map
                    if let Ok(uuid) = player.get_uuid() {
                        uuids_ref.write().remove(&uuid);
                        info!("Player removed from players map");
                    }

                    Ok(())
                } else {
                    error!("Failed to get player username");
                    Ok(())
                }
            } else {
                error!("Failed to get player from event");
                Ok(())
            }
        })?;

        event_node.listen(move |event: &ServerListPingEvent| {
            let response_data = event.get_response_data()?;

            response_data.set_online(-1)?;
            response_data.set_max_player(i32::MAX)?;
            response_data.set_description(&component!("Henlo").red())?;
            response_data.set_favicon(&crate::favicon::random_image())?;

            Ok(())
        })?;

        let states_ref = self.player_states.clone();
        event_node.listen(move |spawn_event: &PlayerMoveEvent| {
            if let Ok(player) = spawn_event.player() {
                if let Ok(name) = player.get_username() {
                    if let Some(state) = states_ref.lock().unwrap().get_mut(&name) {
                        let pos = player.get_position()?;
                        if pos.y < START_POS.1 as f64 - 32.0 {
                            reset_player(&player, state)?;
                        } else {
                            manage_blocks(&player, &pos, state)?;
                        }
                    }
                }
            }
            Ok(())
        })?;

        Ok(())
    }
}

fn create_empty_instance(server: &MinestomServer) -> minestom::Result<InstanceContainer> {
    let instance_manager = server.instance_manager()?;
    let instance = instance_manager.create_instance_container()?;
    Ok(instance)
}

fn reset_player(player: &minestom::entity::Player, state: &mut GameState) -> minestom::Result<()> {
    // Clear existing blocks
    for block in state.blocks.iter() {
        state
            .instance
            .set_block(block.x, block.y, block.z, BlockType::Air.to_block()?)?;
    }
    state.blocks.clear();

    // Reset state
    state
        .blocks
        .push_back(Vec3::new(START_POS.0, START_POS.1, START_POS.2));
    state.score = 0;
    state.combo = 0;
    state.last_block_timestamp = 0;

    // Teleport player
    player.teleport(
        START_POS.0 as f64 + 0.5,
        START_POS.1 as f64 + 10.0,
        START_POS.2 as f64 + 0.5,
        -90.0,
        0.0,
    )?;

    // Place initial block
    let mut rng = rand::thread_rng();
    let block_type = BLOCK_TYPES[rng.gen_range(0..BLOCK_TYPES.len())];
    state.instance.set_block(
        START_POS.0,
        START_POS.1,
        START_POS.2,
        block_type.to_block()?,
    )?;

    // Generate initial blocks
    for _ in 1..10 {
        generate_next_block(state, false)?;
    }

    Ok(())
}

fn generate_random_block(pos: &Vec3, target_y: i32) -> Vec3 {
    let mut rng = rand::rng();
    let y = rng.gen_range(-1..=1);
    let z = if y == 1 {
        rng.gen_range(1..=2)
    } else {
        rng.gen_range(2..=4)
    };
    let x = rng.gen_range(-3..=3);
    Vec3::new(pos.x + x, pos.y + y, pos.z + z)
}

fn generate_next_block(state: &mut GameState, in_game: bool) -> minestom::Result<()> {
    if in_game {
        if let Some(removed_block) = state.blocks.pop_front() {
            state.instance.set_block(
                removed_block.x,
                removed_block.y,
                removed_block.z,
                BlockType::Air.to_block()?,
            )?;
            state.score += 1;
        }
    }

    let last_pos = state.blocks.back().unwrap().clone();
    let block_pos = generate_random_block(&last_pos, state.target_y);

    let mut rng = rand::thread_rng();
    let block_type = BLOCK_TYPES[rng.gen_range(0..BLOCK_TYPES.len())];
    state.instance.set_block(
        block_pos.x,
        block_pos.y,
        block_pos.z,
        block_type.to_block()?,
    )?;
    state.blocks.push_back(block_pos);

    state.last_block_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    Ok(())
}

fn manage_blocks(
    player: &minestom::entity::Player,
    pos: &Position,
    state: &mut GameState,
) -> minestom::Result<()> {
    let block_under_player = Vec3::new(
        (pos.x - 0.5).round() as i32,
        (pos.y - 1.0).floor() as i32,
        (pos.z - 0.5).round() as i32,
    );

    if let Some(index) = state.blocks.iter().position(|b| {
        b.x == block_under_player.x && b.y == block_under_player.y && b.z == block_under_player.z
    }) {
        if index > 0 {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            let max_time_taken =
                (1000.0 * index as f64 / (2.0f64.powf(state.combo as f64 / 45.0))) as u64;

            if current_time - state.last_block_timestamp < max_time_taken {
                state.combo += index as i32;
            } else {
                state.combo = 0;
            }

            for _ in 0..index {
                generate_next_block(state, true)?;
            }

            let pitch = 0.9 + (state.combo - 1) as f32 * 0.05;
            player.play_sound(&Sound::sound(
                SoundEvent::BlockNoteBlockBass,
                Source::Record,
                1.0,
                pitch,
            )?)?;

            let msg = component!("Current score: {}", state.score);
            player.send_message(&msg)?;
        }
    }

    Ok(())
}

pub async fn run_server() -> minestom::Result<()> {
    init_logging();

    let minecraft_server = MinestomServer::new()?;

    let player_states: Arc<Mutex<HashMap<String, GameState>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let event_handler = minecraft_server.event_handler()?;
    let server_ref = minecraft_server.clone();
    let states_ref = player_states.clone();

    event_handler.listen(move |config_event: &AsyncPlayerConfigurationEvent| {
        info!("Setting spawning instance for player");

        if let Ok(player) = config_event.player() {
            if let Ok(name) = player.get_username() {
                info!("Creating empty instance and game state for player");
                let instance = create_empty_instance(&server_ref)?;
                let game_state = GameState::new(instance.clone());
                states_ref.lock().unwrap().insert(name.clone(), game_state);
                config_event.spawn_instance(&instance)?;
            }
        }

        Ok(())
    })?;

    info!("Starting server on 0.0.0.0:25565...");
    minecraft_server.start("0.0.0.0", 25565)?;

    info!("Server is now listening for connections!");
    info!("Press Ctrl+C to stop the server");

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
