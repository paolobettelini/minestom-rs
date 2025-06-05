use minestom::Block;
use minestom::BlockType;
use minestom::InstanceContainer;
use minestom::Player;
use minestom::PlayerMoveEvent;
use minestom::Pos;
use minestom::entity::EntityCreature;
use minestom::entity::ItemDisplay;
use minestom::entity::MinestomEntityCreature;
use minestom::entity::create_entity_creature;
use minestom::entity::entity::EntityType;
use minestom::event::player::{PlayerDisconnectEvent, PlayerSpawnEvent};
use minestom::instance::InstanceManager;
use minestom::item::ItemStack;
use minestom::material::Material;
use parking_lot::RwLock;
use rand::Rng;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use uuid::Uuid;
use world_seed_entity_engine::animation_handler::AnimationHandler;
use world_seed_entity_engine::generic_model::GenericModel;
use world_seed_entity_engine::generic_model::WseeModel;
use world_seed_entity_engine::generic_model::create_wsee_model;

#[derive(Clone)]
pub struct BulbasaurModel;
impl GenericModel for BulbasaurModel {
    fn get_id(&self) -> String {
        "oldman/oldman.bbmodel".to_string()
    }

    fn init(&self, instance: InstanceContainer, pos: Pos) {}
}
