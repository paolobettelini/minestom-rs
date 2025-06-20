use minestom::{
    self, Command,
    command::{CommandContext, CommandSender},
    component,
};

#[derive(Debug, Clone)]
pub struct WebloginCommand;

impl WebloginCommand {
    pub fn register(
        self,
        command_manager: &minestom::command::CommandManager,
    ) -> minestom::Result<()> {
        let builder = command_manager.register(self)?;

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
