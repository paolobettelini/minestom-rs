use log::info;
use minestom::{
    self, Command, Player,
    command::{CommandContext, CommandSender},
    component,
};
use parking_lot::RwLock;
use std::{any::Any, collections::HashMap, sync::Arc};
use thecrown_common::{
    map::LobbyMap,
    player::GetGameServer,
    server::{ArcServerDowncast, Server},
};
use uuid::Uuid;

use crate::{LobbyServer, maps::LobbyMap2};

#[derive(Debug, Clone)]
pub struct SpawnCommand;

impl SpawnCommand {
    pub fn register(
        self,
        command_manager: &minestom::command::CommandManager,
    ) -> minestom::Result<()> {
        let builder = command_manager.register(self)?;

        // Add a condition that checks if the player is in the hashmap
        builder.set_condition(move |sender| {
            if let Ok(player) = sender.as_player() {
                if let Some(server) = player.get_server() {
                    return Ok(server.downcast_ref::<LobbyServer<LobbyMap2>>().is_some());
                }
            }
            Ok(false)
        })?;

        Ok(())
    }
}

impl Command for SpawnCommand {
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
        if let Some(server) = player.get_server() {
            if let Some(lobby) = server.downcast_ref::<LobbyServer<LobbyMap2>>() {
                let (x, y, z, yaw, pitch) = lobby.map.spawn_coordinate();
                player.teleport(x, y, z, yaw, pitch)?;
            }
        }

        player.send_message(&message)?;

        Ok(())
    }
}
