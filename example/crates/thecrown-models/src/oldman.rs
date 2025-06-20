use minestom::{Pos, instance::Instance};
use world_seed_entity_engine::generic_model::GenericModel;

#[derive(Clone)]
pub struct OldManModel;
impl GenericModel for OldManModel {
    fn get_id(&self) -> String {
        "oldman/oldman.bbmodel".to_string()
    }

    fn init(&self, instance: &dyn Instance, pos: Pos) {}
}
