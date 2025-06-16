use std::sync::Arc;

use minestom::{
    self, Command, TOKIO_HANDLE,
    command::{CommandContext, CommandSender},
    component, Suggestion, SuggestionEntry, create_string_arg, create_greedy_string_arg,
};
use thecrown_common::nats::NatsClient;
use thecrown_protocol::RelayPacket;

#[derive(Debug, Clone)]
pub struct WhisperCommand {
    nats_client: Arc<NatsClient>,
}

impl WhisperCommand {
    pub fn new(nats_client: Arc<NatsClient>) -> Self {
        Self { nats_client }
    }
    pub fn register(
        self,
        command_manager: &minestom::command::CommandManager,
    ) -> minestom::Result<()> {
        let builder = command_manager.register(self)?;

        // Add syntax: /whisper <player> <message>
        let player_arg = create_string_arg("player")?;
        let message_arg = create_greedy_string_arg("message")?;

        // Set suggestion callback for player names
        player_arg.set_suggestion_callback(|_sender, _context, suggestion| {
            let player1 = SuggestionEntry::new("Alice")?;
            let player2 = SuggestionEntry::new("Bob")?;
            
            suggestion.add_entry(&player1)?;
            suggestion.add_entry(&player2)?;
            
            Ok(())
        })?;

        builder.add_syntax_with_args(&[&player_arg, &message_arg])?;

        // Set condition to only allow players
        builder.set_condition(move |sender| Ok(sender.is_player()?))?;

        Ok(())
    }
}

impl Command for WhisperCommand {
    fn name(&self) -> &str {
        "whisper"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["w"]
    }

    fn execute(&self, sender: &CommandSender, context: &CommandContext) -> minestom::Result<()> {
        let player = sender.as_player()?;

        // Get the target player and message from arguments
        let message = context.get_string_array("message")?.join(" ");
        let target = context.get_string("player")?;

        if message.trim().is_empty() {
            let error_msg = component!("You must provide a message to whisper!").red();
            player.send_message(&error_msg)?;
            return Ok(());
        }

        // Get player names for the whisper message
        let sender = player.get_username()?;

        let nats = self.nats_client.clone();
        let response = TOKIO_HANDLE.block_on(async {
            let packet = RelayPacket::WhisperCommand {
                sender,
                target: target.clone(),
                message: message.clone(),
            };
            nats.request(&packet).await
        });

        if let Some(RelayPacket::WhisperCommandResponse { status }) = response {
            let msg = if status {
                component!("Whisper sent to ")
                    .gray()
                    .chain(component!("{}", target).yellow())
                    .chain(component!(": ").gray())
                    .chain(component!("{}", message).white())
            } else {
                component!("Player {} is not online!", target)
                    .red()
            };

          player.send_message(&msg)?;
        }


        Ok(())
    }
}
