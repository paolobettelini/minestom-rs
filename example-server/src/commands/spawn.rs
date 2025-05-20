use crate::maps::LobbyMap;
use log::info;
use minestom::{
    Command,
    command::{self, CommandContext, CommandSender},
    component,
};
use minestom_rs as minestom;

#[derive(Debug, Clone)]
pub struct SpawnCommand<T: LobbyMap> {
    pub map: T,
}

impl<T: LobbyMap> SpawnCommand<T> {
    pub fn new(map: T) -> Self {
        Self { map }
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
