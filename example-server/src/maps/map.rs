pub trait LobbyMap: Copy + Clone + Send + Sync + 'static {
    fn anvil_path(&self) -> String;

    fn spawn_coordinate(&self) -> (f64, f64, f64, f32, f32);
}
