use minestom::{
    instance::{Instance, InstanceContainer}, Player, Pos,
    entity::{EntityCreature, MinestomEntityCreature, create_entity_creature, entity::EntityType},
};
use std::sync::{Arc, Mutex, Weak};
use world_seed_entity_engine::{
    animation_handler::AnimationHandler,
    generic_model::{GenericModel, WseeModel, create_wsee_model},
};

#[derive(Clone)]
pub struct BulbasaurModel;
impl GenericModel for BulbasaurModel {
    fn get_id(&self) -> String {
        "bulbasaur/bulbasaur.bbmodel".to_string()
    }

    fn init(&self, instance: &dyn Instance, pos: Pos) {}
}

pub struct BulbasaurMob {
    creature_handle: Weak<Mutex<MinestomEntityCreature>>,
    model: WseeModel,
    instance: InstanceContainer,
    spawn_pos: Pos,
    animation_handler: AnimationHandler,
}

impl BulbasaurMob {
    pub fn new(
        instance: &dyn Instance,
        spawn_pos: Pos,
    ) -> minestom::Result<Arc<Mutex<MinestomEntityCreature>>> {
        let placeholder: Arc<Mutex<MinestomEntityCreature>> =
            Arc::new(Mutex::new(MinestomEntityCreature::null()));

        let model = BulbasaurModel;
        let model = create_wsee_model(model)?;

        let animation_handler = AnimationHandler::new(&model)?;

        // Convert to InstanceContainer for storage
        let instance_container = match instance.inner() {
            Ok(obj) => InstanceContainer::new(minestom::jni_utils::JavaObject::from_env(&mut minestom::jni_utils::get_env()?, obj)?),
            Err(_) => return Err(minestom::MinestomError::EventError("Failed to get instance".to_string())),
        };

        let mob_impl = Self {
            creature_handle: Arc::downgrade(&placeholder),
            model: model.clone(),
            instance: instance_container.clone(),
            spawn_pos: spawn_pos.clone(),
            animation_handler: animation_handler.clone(),
        };

        let mob_impl_arc: Arc<dyn EntityCreature> = Arc::new(mob_impl);

        let wrapper = create_entity_creature(EntityType::Zombie, mob_impl_arc.clone())?;
        {
            let mut guard = placeholder.lock().unwrap();
            *guard = wrapper.clone();
        }

        wrapper.set_invisible(true)?;

        model.init(&instance_container, spawn_pos.clone())?;
        let _ = animation_handler.play_repeat("animation.bulbasaur.faint");

        wrapper.set_instance_and_pos(&instance_container, &spawn_pos)?;

        Ok(placeholder.clone())
    }
}

impl EntityCreature for BulbasaurMob {
    fn update_new_viewer(&self, player: Player) {
        let _ = self.model.add_viewer(&player);
    }

    fn update_old_viewer(&self, player: Player) {
        let _ = self.model.remove_viewer(&player);
    }

    fn tick(&self, time: i64) {}

    fn remove(&self) {
        // TODO: model,animation_handler.destroy()
    }
}
