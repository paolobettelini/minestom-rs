use minestom::{InstanceContainer, SharedInstance, Player};
use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

pub trait LobbyMap: Clone + Send + Sync + 'static {
    fn spawn_coordinate(&self) -> (f64, f64, f64, f32, f32);

    fn init(&self, players: Arc<RwLock<HashMap<Uuid, Player>>>) -> minestom::Result<()>;

    fn instance(&self) -> SharedInstance;
}

/// Function to create an InstanceContainer for a lobby map.
/// This should be called from the mcserver to create the underlying instance.
/// 
/// This function creates a single InstanceContainer that loads the lobby world data.
/// From this container, multiple SharedInstances can be created to allow multiple
/// lobbies to share the same world without duplicating the expensive world data.
/// 
/// # Example Usage
/// ```rust,no_run
/// // In mcserver - create once
/// let instance_container = create_lobby_instance_container(&instance_manager)?;
/// 
/// // Create multiple shared instances for different lobbies
/// let lobby1_shared = instance_container.create_shared_instance()?;
/// let lobby2_shared = instance_container.create_shared_instance()?;
/// 
/// // Each lobby uses its own SharedInstance but shares the same world data
/// let lobby1 = LobbyMap2::new(lobby1_shared)?;
/// let lobby2 = LobbyMap2::new(lobby2_shared)?;
/// ```
pub fn create_lobby_instance_container(instance_manager: &minestom::instance::InstanceManager) -> minestom::Result<InstanceContainer> {
    let anvil_path = String::from("/home/paolo/Desktop/github/minestom-rs/example/resources/anvil/lobby2");
    let instance = instance_manager.create_instance_container()?;
    instance.load_anvil_world(anvil_path)?;
    Ok(instance)
}
