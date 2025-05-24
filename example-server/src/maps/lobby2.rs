use crate::maps::LobbyMap;
use minestom_rs::InstanceContainer;
use minestom_rs::PlayerMoveEvent;
use minestom_rs::instance::InstanceManager;
use rand::Rng;
use std::sync::Arc;

#[derive(Clone)]
pub struct LobbyMap2 {
    pub instance: Arc<InstanceContainer>,
}

impl LobbyMap2 {
    pub fn new(instance_manager: &InstanceManager) -> minestom_rs::Result<Self> {
        let anvil_path =
            String::from("/home/paolo/Desktop/github/minestom-rs/example-server/anvil/lobby2");
        let instance = instance_manager.create_instance_container()?;
        let instance = Arc::new(instance);
        instance.load_anvil_world(anvil_path)?;
        Ok(Self {
            instance: instance.clone(),
        })
    }
}

impl LobbyMap for LobbyMap2 {
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

    fn init(&self) -> minestom_rs::Result<()> {
        let event_node = self.instance.event_node()?;


        log::info!("AAAAAAAAAAAAAAAAAAAAAAAAAAAAa");

        let map = self.clone();
        event_node.listen(move |move_event: &PlayerMoveEvent| {
            if let Ok(player) = move_event.player() {
                if let Ok(pos) = player.get_position() {
                    if pos.y < 0.0 {
                        log::info!("XXXXXXXXXXXXX");
                        let (x, y, z, yaw, pitch) = map.spawn_coordinate();
                        player.teleport(x, y, z, yaw, pitch)?;
                    }
                }
            }
            Ok(())
        })?;

        Ok(())
    }

    fn instance(&self) -> Arc<InstanceContainer> {
        self.instance.clone()
    }
}
