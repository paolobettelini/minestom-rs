use crate::commands::SpawnCommand;
use crate::logic::piano;
use crate::magic_values::*;
use crate::maps::LobbyMap2;
use crate::maps::map::LobbyMap;
use crate::mojang::get_skin_and_signature;
use crate::server::Server;
use log::{error, info};
use minestom::MinestomServer;
use minestom_rs as minestom;
use minestom_rs::InstanceContainer;
use minestom_rs::Player;
use minestom_rs::ServerListPingEvent;
use minestom_rs::TOKIO_HANDLE;
use minestom_rs::entity::PlayerSkin;
use minestom_rs::{
    attribute::Attribute,
    command::{Command, CommandContext},
    component,
    entity::GameMode,
    entity::ItemDisplay,
    event::player::{
        AsyncPlayerConfigurationEvent, PlayerChatEvent, PlayerDisconnectEvent, PlayerMoveEvent,
        PlayerSkinInitEvent, PlayerSpawnEvent,
    },
    item::{InventoryHolder, ItemStack},
    material::Material,
    resource_pack::{ResourcePackInfo, ResourcePackRequest, ResourcePackRequestBuilder},
};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct LobbyServer<T: LobbyMap> {
    map: T,
    players: Arc<RwLock<HashMap<Uuid, Player>>>,
}

impl<T: LobbyMap> LobbyServer<T> {
    pub fn new(map: T) -> minestom::Result<Self> {
        let players = Arc::new(RwLock::new(HashMap::new()));
        map.init(players.clone())?;
        Ok(LobbyServer { map, players })
    }
}

impl<T: LobbyMap> Server for LobbyServer<T> {
    fn init_player(
        &self,
        minecraft_server: &MinestomServer,
        config_event: &AsyncPlayerConfigurationEvent,
    ) -> minestom::Result<()> {
        if let Ok(player) = config_event.player() {
            info!("Setting spawning instance for player");
            config_event.spawn_instance(&self.map.instance())?;
        }

        Ok(())
    }

    fn init(&self, minecraft_server: &MinestomServer) -> minestom::Result<()> {
        let scheduler = minecraft_server.scheduler_manager()?;
        let instance_manager = minecraft_server.instance_manager()?;
        let command_manager = minecraft_server.command_manager()?;

        // TODO use ShareInstance from a static instance
        let instance = self.map.instance();

        let event_handler = instance.event_node()?;
        let spawn_instance = instance.clone();

        let players = self.players.clone();

        // Register commands
        let spawn_command = SpawnCommand::new(self.map.clone(), players.clone());
        spawn_command.register(&command_manager)?;

        let map_clone = self.map.clone();
        event_handler.listen(move |spawn_event: &PlayerSpawnEvent| {
            info!("Player spawn event triggered");
            if let Ok(player) = spawn_event.player() {
                let username = player.get_username()?;

                // Add player to the players map
                let uuid = player.get_uuid()?;
                players.write().insert(uuid, player.clone());

                let welcome_msg = component!("Welcome to the server, {}!", username)
                    .gold()
                    .bold();
                let info_msg = component!("Enjoy your adventure!").green().italic();
                let message = welcome_msg.chain_newline(info_msg);

                player.send_message(&message)?;
                player.set_game_mode(GameMode::Adventure)?;

                let (x, y, z, yaw, pitch) = map_clone.spawn_coordinate();
                player.teleport(x, y, z, yaw, pitch)?;
                player.set_allow_flying(true)?;

                let scale = distribution(AVG_SCALE, MIN_SCALE, MAX_SCALE);
                info!("Setting player scale to {}", scale);
                player
                    .get_attribute(Attribute::Scale)?
                    .set_base_value(scale)?;
                player
                    .get_attribute(Attribute::JumpStrength)?
                    .set_base_value(jump_strength_scale(scale))?;
                player
                    .get_attribute(Attribute::StepHeight)?
                    .set_base_value(step_height_scale(scale))?;

                // Send [+] join message to everyone
                let players = players.read();
                for player in players.values() {
                    let msg = component!("[")
                        .color("#454545")?
                        .chain(component!("+").green())
                        .chain(component!("] ").color("#454545")?)
                        .chain(component!("{}", username).color("#669999")?)
                        .chain(component!(" joined the game.").color("#ebebeb")?);

                    player.send_message(&msg)?;
                }

                // Get player's inventory and set the helmet
                let item =
                    ItemStack::of(Material::BoltArmorTrimSmithingTemplate)?.with_amount(1)?;
                let inventory = player.get_inventory()?;
                inventory.set_helmet(&item)?;

                // refresh condition so that the player can list commands
                player.refresh_commands()?;
            }
            Ok(())
        })?;

        // Handle player disconnect
        let players_disconnect = self.players.clone();
        event_handler.listen(move |event: &PlayerDisconnectEvent| {
            if let Ok(player) = event.player() {
                if let Ok(uuid) = player.get_uuid() {
                    players_disconnect.write().remove(&uuid);
                    info!("Player disconnected and removed from players map");
                }
            }
            Ok(())
        })?;

        // Handle chat messages
        let players_ref = self.players.clone();
        event_handler.listen(move |event: &PlayerChatEvent| {
            event.set_cancelled(true)?;

            let player = event.player()?;
            let raw_msg = event.raw_message()?;
            let username = player.get_username()?;
            let formatted = component!("[{}] {}", username, raw_msg);

            // Send to all players
            let players = players_ref.read();
            for player in players.values() {
                player.send_message(&formatted)?;
            }

            Ok(())
        })?;

        Ok(())
    }
}
