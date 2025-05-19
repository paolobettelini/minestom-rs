use log::info;
use minestom::{
    Command,
    command::{self, CommandContext, CommandSender},
    component,
};
use minestom_rs as minestom;

#[derive(Debug, Clone)]
pub struct SpawnCommand {
    spawn_x: f64,
    spawn_y: f64,
    spawn_z: f64,
    spawn_yaw: f32,
    spawn_pitch: f32,
}

impl SpawnCommand {
    pub fn new(spawn_x: f64, spawn_y: f64, spawn_z: f64, spawn_yaw: f32, spawn_pitch: f32) -> Self {
        Self {
            spawn_x,
            spawn_y,
            spawn_z,
            spawn_yaw,
            spawn_pitch,
        }
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

        // Create a gold, italic text component
        let message = component!("Welcome to spawn!").gold().italic();

        let player = sender.as_player()?;
        player.send_message(&message)?;
        player.teleport(
            self.spawn_x,
            self.spawn_y,
            self.spawn_z,
            self.spawn_yaw,
            self.spawn_pitch,
        )?;

        Ok(())
    }
}
