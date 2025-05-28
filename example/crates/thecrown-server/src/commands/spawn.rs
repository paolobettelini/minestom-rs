use crate::maps::LobbyMap;
use log::info;
use minestom::{
    Command,
    command::{self, CommandContext, CommandSender},
    component,
};
use minestom as minestom;
use minestom::Player;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SpawnCommand<T: LobbyMap> {
    pub map: T,
    pub players: Arc<RwLock<HashMap<Uuid, Player>>>,
}

impl<T: LobbyMap> SpawnCommand<T> {
    pub fn new(map: T, players: Arc<RwLock<HashMap<Uuid, Player>>>) -> Self {
        Self { map, players }
    }

    pub fn register(
        &self,
        command_manager: &minestom::command::CommandManager,
    ) -> minestom::Result<()> {
        let builder = command_manager.register(self.clone())?;

        // Clone the map and players for use in the closure
        let map = self.map.clone();
        let players = self.players.clone();

        // Add a condition that checks if the player is in the hashmap
        builder.set_condition(move |sender| {
            if let Ok(player) = sender.as_player() {
                let players_guard = players.read();
                if players_guard.contains_key(&player.get_uuid()?) {
                    return Ok(true);
                }
            }
            Ok(false)
        })?;

        Ok(())
    }
}

impl<T: LobbyMap> Command for SpawnCommand<T> {
    fn name(&self) -> &str {
        "spawn"
    }

    fn aliases(&self) -> Vec<&str> {
        vec![]
    }

    fn execute(&self, sender: &CommandSender, _context: &CommandContext) -> minestom::Result<()> {
        info!("Player used the spawn command!");

        let message = component!("Welcome to spawn!").gold().italic();

        let player = sender.as_player()?;
        player.send_message(&message)?;
        let (x, y, z, yaw, pitch) = self.map.spawn_coordinate();
        player.teleport(x, y, z, yaw, pitch)?;

        Ok(())
    }
}
