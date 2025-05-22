use minestom_rs::InstanceContainer;

pub trait LobbyMap: Clone + Send + Sync + 'static {
    fn spawn_coordinate(&self) -> (f64, f64, f64, f32, f32);

    fn init(&self, instance: &InstanceContainer) -> minestom_rs::Result<()>;

    fn instance(&self) -> InstanceContainer;
}
