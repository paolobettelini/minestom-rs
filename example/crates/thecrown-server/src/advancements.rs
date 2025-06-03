use crate::commands::SpawnCommand;
use crate::logic::lobby::LobbyServer;
use minestom::advancement::FrameType;
use crate::logic::parkour::ParkourServer;
use crate::magic_values::*;
use minestom::advancement::Advancement;
use minestom::advancement::AdvancementRoot;
use minestom::advancement::AdvancementManager;
use crate::maps::LobbyMap2;
use crate::maps::map::LobbyMap;
use crate::mojang::get_skin_and_signature;
use log::{error, info};
use minestom;
use minestom::InstanceContainer;
use minestom::MinestomServer;
use minestom::ServerListPingEvent;
use minestom::TOKIO_HANDLE;
use minestom::entity::PlayerSkin;
use minestom::{
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
use rand::Rng;
use std::collections::HashMap;
use minestom::Player;
use std::path::Path;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::{Mutex, RwLock};
use uuid::Uuid;
use world_seed_entity_engine::model_engine::ModelEngine;

static ADVANCEMENTS: LazyLock<RwLock<HashMap<Uuid, ThecrownAdvancements>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub struct ThecrownAdvancements {
    pub advancements: HashMap<String, Advancement>,
}

impl ThecrownAdvancements {
    pub fn new(minecraft_server: &MinestomServer, player: &Player) -> minestom::Result<Self> {
        let adv_manager = minecraft_server.advancement_manager()?;
        let root = AdvancementRoot::new(
            &component!("Welcome!"),
            &component!("Join the server!"),
            Material::NetherStar,
            FrameType::TASK(),
            0.0,
            0.0,
            Some("minecraft:textures/block/stone.png"),
        )?;
        let _ = root.as_advancement().set_achieved(true); // Achieved by default
        let tab = adv_manager.create_tab("thecrown", root.clone())?;
        tab.add_viewer(&player)?;

        let mut advancements = HashMap::new();

        let name = "thecrown/honeyishrunkmyself";
        let honey_i_shrunk_myself = Advancement::new(
            &component!("Honey, I shrunk myself!"),
            &component!("You completed the first objective"),
            Material::GoldIngot,
            FrameType::GOAL(),
            1.0,
            1.0,
        )?;
        let _ = honey_i_shrunk_myself.show_toast(true);
        tab.create_advancement(name, honey_i_shrunk_myself.clone(), root.clone().as_advancement())?;

        advancements.insert(name.to_string(), honey_i_shrunk_myself);

        Ok(Self { advancements })
    }
}

pub fn init_player_advancements(minecraft_server: &MinestomServer, player: &Player) -> minestom::Result<()> {
    let uuid = player.get_uuid()?;
    let advancements = ThecrownAdvancements::new(minecraft_server, player)?;

    // add to the map
    let mut player_advancements = ADVANCEMENTS.write().unwrap();
    player_advancements.insert(uuid, advancements);
    Ok(())
}

pub fn get_advancement(player: &Player, name: &str) -> Option<Advancement> {
    let advancements = ADVANCEMENTS.read().unwrap();
    if let Some(player_advancements) = advancements.get(&player.get_uuid().ok()?) {
        player_advancements.advancements.get(name).cloned()
    } else {
        None
    }
} 