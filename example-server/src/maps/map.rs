use minestom_rs::InstanceContainer;
use std::sync::Arc;

pub trait LobbyMap: Clone + Send + Sync + 'static {
    fn spawn_coordinate(&self) -> (f64, f64, f64, f32, f32);

    fn init(&self) -> minestom_rs::Result<()>;

    fn instance(&self) -> Arc<InstanceContainer>;
}
