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
use world_seed_entity_engine::generic_model::GenericModel;
use world_seed_entity_engine::generic_model::WseeModel;
use world_seed_entity_engine::generic_model::create_wsee_model;

#[derive(Clone)]
pub struct BulbasaurModel;
impl GenericModel for BulbasaurModel {
    fn get_id(&self) -> String {
        "bulbasaur/bulbasaur.bbmodel".to_string()
    }

    fn init(&self, instance: InstanceContainer, pos: Pos) {}
}

pub struct BulbasaurMob {
    creature_handle: Weak<Mutex<MinestomEntityCreature>>,
    model: WseeModel,
    instance: InstanceContainer,
    spawn_pos: Pos,
}

impl BulbasaurMob {
    pub fn new(
        instance: InstanceContainer,
        spawn_pos: Pos,
    ) -> minestom::Result<Arc<Mutex<MinestomEntityCreature>>> {
        let placeholder: Arc<Mutex<MinestomEntityCreature>> =
            Arc::new(Mutex::new(MinestomEntityCreature::null()));

        let model = BulbasaurModel;
        let model = create_wsee_model(model)?;

        let mob_impl = Self {
            creature_handle: Arc::downgrade(&placeholder),
            model: model.clone(),
            instance: instance.clone(),
            spawn_pos: spawn_pos.clone(),
        };

        let mob_impl_arc: Arc<dyn EntityCreature> = Arc::new(mob_impl);

        let wrapper = create_entity_creature(EntityType::Zombie, mob_impl_arc.clone())?;
        {
            let mut guard = placeholder.lock().unwrap();
            *guard = wrapper.clone();
        }

        wrapper.set_invisible(true)?;

        wrapper.set_instance_and_pos(&instance, &spawn_pos)?;
        model.init(instance.clone(), spawn_pos)?;

        Ok(placeholder.clone())
    }
}

impl EntityCreature for BulbasaurMob {
    fn update_new_viewer(&self, player: Player) {
        log::info!("ARE YOU READY TO SEE THE BULBASAUR?");
        let _ = self.model.add_viewer(&player);
    }

    fn update_old_viewer(&self, player: Player) {
        let _ = self.model.remove_viewer(&player);
    }

    fn tick(&self, time: i64) {}

    fn remove(&self) {}
}
