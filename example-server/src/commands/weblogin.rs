use crate::maps::LobbyMap;
use log::info;
use minestom::{
    Command,
    command::{self, CommandContext, CommandSender},
    component,
};
use minestom_rs as minestom;
use minestom_rs::Player;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct WebloginCommand;

impl WebloginCommand {
    pub fn register(
        &self,
        command_manager: &minestom::command::CommandManager,
    ) -> minestom::Result<()> {
        let builder = command_manager.register(self.clone())?;

        builder.set_condition(move |_| Ok(true))?;

        Ok(())
    }
}

impl Command for WebloginCommand {
    fn name(&self) -> &str {
        "weblogin"
    }

    fn aliases(&self) -> Vec<&str> {
        vec![]
    }

    fn execute(&self, sender: &CommandSender, _context: &CommandContext) -> minestom::Result<()> {
        let message = component!("This feature is not ready!").gold().italic();
        let player = sender.as_player()?;

        player.send_message(&message)?;

        Ok(())
    }
}
