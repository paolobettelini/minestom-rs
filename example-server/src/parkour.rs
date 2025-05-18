use log::{debug, error, info};
use minestom::{MinestomServer, Block};
use minestom::{
    component,
    entity::GameMode,
    event::{
        Event,
        player::{AsyncPlayerConfigurationEvent, PlayerSpawnEvent},
    },
    sound::{Sound, SoundEvent, Source},
    text::Component,
};
use minestom_rs as minestom;

struct GameState {

}

impl GameState {
    fn new() -> Self {
        todo!()
    }
}

pub async fn run_server() -> minestom::Result<()> {
    init_logging();

    let anvil_path = "/home/paolo/Desktop/github/minestom-rs/example-server/anvil";
    let start_pos = (0, 100, 0);
    let (spawn_yaw, spawn_pitch) = (-90.0, 0.0);

    let minecraft_server = MinestomServer::new()?;
    let instance_manager = minecraft_server.instance_manager()?;
    let instance = instance_manager.create_instance_container()?;
    instance.load_anvil_world(anvil_path)?;

    let event_handler = minecraft_server.event_handler()?;
    let spawn_instance = instance.clone();

    // TODO: make a hashmap of players and their game state

    event_handler.register_event_listener(
        move |config_event: &AsyncPlayerConfigurationEvent| {
            info!("Setting spawning instance for player");

            // Try to get player information
            if let Ok(player) = config_event.player() {
                if let Ok(name) = player.get_username() {
                    // TODO: add the player to the hashmap with a new game state
                    // TODO: set the instance to game_state.instance
                    // config_event.spawn_instance(&spawn_instance)?;
                }
            }

            Ok(())
        },
    )?;

    let welcome_instance = instance.clone();
    event_handler.register_event_listener(move |spawn_event: &PlayerSpawnEvent| {
        info!("Player spawn event triggered");
        if let Ok(player) = spawn_event.player() {
            let username = player.get_username()?;

            // TODO: teleport the player to START_POS
            // player.teleport(spawn_x, spawn_y, spawn_z, spawn_yaw, spawn_pitch)?;

            spawn_instance.set_block(
                spawn_x as i32,
                spawn_y as i32 + 3,
                spawn_z as i32,
                Block::Stone,
            )?;

            player.set_game_mode(GameMode::Adventure)?;

            // call resetPlayer(player, gameState)
        }
        Ok(())
    })?;

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

// TODO: make a function reset player which:
// for every block in state.blocks, set it to Block.AIR
// clear the blocks in the state
// state.blocks.add(new Vec(START_POS.x(), START_POS.y(), START_POS.z()));
// state.score = 0;
// state.combo = 0;
// state.lastBlockTimestamp = 0;
// Teleport to START_POS + (0.5, 10.0, 0.5)
// Convert this code from java
// state.instance.setBlock(START_POS, BLOCK_TYPES[new Random().nextInt(BLOCK_TYPES.length)]);
// state.blocks.add(new Vec(START_POS.x(), START_POS.y(), START_POS.z()));
// for (int i = 1; i < 10; i++) {
//     generateNextBlock(state, false);
// }
// 
// private static void manageBlocks(Player player, Pos pos, GameState state) {
//     Vec blockUnderPlayer = new Vec(
//             Math.round(pos.x() - 0.5),
//             Math.floor(pos.y() - 1),
//             Math.round(pos.z() - 0.5)
//     );
// 
//     if (state.blocks.contains(blockUnderPlayer)) {
//         List<Vec> blockList = new ArrayList<>(state.blocks);
//         int index = blockList.indexOf(blockUnderPlayer);
//         if (index > 0) {
//             long currentTimeMillis = Instant.now().toEpochMilli();
//             long maxTimeTaken = (long) (1000 * index / Math.pow(2, state.combo / 45.0));
// 
//             if (currentTimeMillis - state.lastBlockTimestamp < maxTimeTaken) {
//                 state.combo += index;
//             } else {
//                 state.combo = 0;
//             }
// 
//             for (int i = 0; i < index; i++) {
//                 generateNextBlock(state, true);
//             }
// 
//             float pitch = 0.9f + (state.combo - 1) * 0.05f;
//             player.playSound(Sound.sound(SoundEvent.BLOCK_NOTE_BLOCK_BASS, Source.RECORD, 1f, pitch));
// 
//             player.sendMessage(Common.formatServerMsg("Current score: " + state.score));
//         }
//     }
// }
// 
// private static void generateNextBlock(GameState state, boolean inGame) {
//     if (inGame) {
//         Vec removedBlock = state.blocks.poll();
//         if (removedBlock != null) {
//             state.instance.setBlock(removedBlock, Block.AIR);
//             state.score++;
//         }
//     }
// 
//     Vec lastPos = state.blocks.peekLast();
//     Vec blockPos = generateRandomBlock(lastPos, state.targetY);
// 
//     state.instance.setBlock(blockPos, BLOCK_TYPES[new Random().nextInt(BLOCK_TYPES.length)]);
//     state.blocks.add(blockPos);
// 
//     state.lastBlockTimestamp = Instant.now().toEpochMilli();
// }
// 
// private static Vec generateRandomBlock(Vec pos, int targetY) {
//     Random rng = new Random();
// 
//     int y;
//     if (targetY == 0) {
//         y = rng.nextInt(3) - 1;
//     } else if (targetY > pos.y()) {
//         y = 1;
//     } else {
//         y = -1;
//     }
// 
//     int z = y == 1 ? rng.nextInt(2) + 1 : rng.nextInt(3) + 2;
//     int x = rng.nextInt(7) - 3;
// 
//     return new Vec(pos.x() + x, pos.y() + y, pos.z() + z);
// }

fn init_logging() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_timestamp(Some(env_logger::fmt::TimestampPrecision::Millis))
        .init();
}
