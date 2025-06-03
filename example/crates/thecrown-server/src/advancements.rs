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
use std::path::Path;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::{Mutex, RwLock};
use uuid::Uuid;
use world_seed_entity_engine::model_engine::ModelEngine;