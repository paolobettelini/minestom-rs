use crate::maps::LobbyMap;
use rand::Rng;

#[derive(Copy, Clone)]
pub struct LobbyMap2;

impl LobbyMap for LobbyMap2 {
    fn anvil_path(&self) -> String {
        String::from("/home/paolo/Desktop/github/minestom-rs/example-server/anvil/lobby2")
    }

    fn spawn_coordinate(&self) -> (f64, f64, f64, f32, f32) {
        let spawns = vec![
            (1794.5, 41.0, 1066.5, 180.0, 0.0),
            (1811.5, 41.0, 1060.5, 135.0, 0.0),
            (1817.5, 41.0, 1044.5, 90.0, 0.0),
            (1811.5, 41.0, 1028.5, 45.0, 0.0),
            (1794.5, 41.0, 1022.5, 0.0, 0.0),
        ];
        // random spawn
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..spawns.len());
        spawns[index]
    }
}
