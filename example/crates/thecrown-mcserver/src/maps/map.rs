use minestom::{InstanceContainer, Player};
use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

pub trait LobbyMap: Clone + Send + Sync + 'static {
    fn spawn_coordinate(&self) -> (f64, f64, f64, f32, f32);

    fn init(&self, players: Arc<RwLock<HashMap<Uuid, Player>>>) -> minestom::Result<()>;

    fn instance(&self) -> InstanceContainer;
}
