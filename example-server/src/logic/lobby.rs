use crate::commands::SpawnCommand;
use crate::magic_values::*;
use crate::maps::LobbyMap2;
use crate::maps::map::LobbyMap;
use crate::mojang::get_skin_and_signature;
use crate::server::Server;
use log::{error, info};
use minestom::MinestomServer;
use minestom_rs as minestom;
use minestom_rs::InstanceContainer;
use minestom_rs::ServerListPingEvent;
use minestom_rs::TOKIO_HANDLE;
use minestom_rs::entity::PlayerSkin;
use minestom_rs::{
    attribute::Attribute,
    command::{Command, CommandContext},
    component,
    entity::GameMode,
    event::player::{
        AsyncPlayerConfigurationEvent, PlayerMoveEvent, PlayerSkinInitEvent, PlayerSpawnEvent,
    },
    item::{InventoryHolder, ItemStack},
    material::Material,
    resource_pack::{ResourcePackInfo, ResourcePackRequest, ResourcePackRequestBuilder},
};
use parking_lot::Mutex as ParkingMutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::{Mutex, RwLock};
use uuid::Uuid;

pub struct LobbyServer<T: LobbyMap> {
    map: T,
}

impl<T: LobbyMap> LobbyServer<T> {
    pub fn new(map: T) -> minestom::Result<Self> {
        Ok(LobbyServer { map })
    }
}

impl<T: LobbyMap> Server for LobbyServer<T> {
    fn init_player(&self, minecraft_server: &MinestomServer, config_event: &AsyncPlayerConfigurationEvent) -> minestom::Result<()> {
        if let Ok(player) = config_event.player() {
            info!("Setting spawning instance for player");
            config_event.spawn_instance(&self.map.instance())?;
        }

        Ok(())
    }

    fn init(&self, minecraft_server: &MinestomServer) -> minestom::Result<()> {
        let scheduler = minecraft_server.scheduler_manager()?;
        let instance_manager = minecraft_server.instance_manager()?;

        // TODO use ShareInstance from a static instance
        let instance = self.map.instance();

        let event_handler = instance.event_node()?;
        let spawn_instance = instance.clone();

        let map_clone = self.map.clone();
        event_handler.listen(move |spawn_event: &PlayerSpawnEvent| {
            info!("Player spawn event triggered");
            if let Ok(player) = spawn_event.player() {
                let username = player.get_username()?;

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

                // https://minecraft.wiki/w/Attribute#Modifiers
                let scale = distribution(AVG_SCALE, MIN_SCALE, MAX_SCALE);
                //let scale = 15.0;
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

                // Create a diamond sombrero
                let sombrero = ItemStack::of(Material::Diamond)?
                    .with_amount(1)?
                    .with_custom_model_data("piano")?;

                // Get player's inventory and set the helmet
                let inventory = player.get_inventory()?;
                inventory.set_helmet(&sombrero)?;
            }
            Ok(())
        })?;

        Ok(())
    }
}
