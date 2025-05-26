use crate::logic::piano;
use crate::maps::LobbyMap;
use minestom_rs::InstanceContainer;
use minestom_rs::BlockType;
use minestom_rs::Player;
use minestom_rs::PlayerMoveEvent;
use minestom_rs::instance::InstanceManager;
use minestom_rs::Block;
use parking_lot::RwLock;
use rand::Rng;
use std::collections::HashMap;
use minestom_rs::material::Material;
use minestom_rs::entity::ItemDisplay;
use minestom_rs::item::ItemStack;
use std::sync::Arc;
use uuid::Uuid;

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

    fn init(&self, players: Arc<RwLock<HashMap<Uuid, Player>>>) -> minestom_rs::Result<()> {
        let event_node = self.instance.event_node()?;

        let map = self.clone();
        event_node.listen(move |move_event: &PlayerMoveEvent| {
            if let Ok(player) = move_event.player() {
                if let Ok(pos) = player.get_position() {
                    if pos.y < 0.0 {
                        let (x, y, z, yaw, pitch) = map.spawn_coordinate();
                        player.teleport(x, y, z, yaw, pitch)?;
                    }
                }
            }
            Ok(())
        })?;

        let block = BlockType::NoteBlock.to_block()?
            .with_property("note", "1")?
            .with_property("powered", "false")?;
        self
            .instance
            .set_block(1761, 35, 1044, block)?;

        // Achievement honey I shrunk myself
        // (1764, 26, 1177) - (1762, 26, 1177)
        // le scritte del cartello non si vedono, nemmeno l'itemframe completamente.

        piano::spawn_piano(self.instance.clone(), players, 1777.4, 28.0, 1056.0, -90.0)?;

        macro_rules! cloud {
            ($name:expr) => {
                ItemStack::of(Material::Diamond)?
                    .with_amount(1)?
                    .with_custom_model_data($name)?
            };
        }

        let clouds = vec![
            cloud!("cloud1"),
            cloud!("cloud2"),
            cloud!("cloud3"),
        ];

        let coords = vec![
            (1725.0, 58.0, 1005.0),
            (1722.0, 57.0, 1008.0),
            (1719.0, 55.0, 1011.0),
            (1716.0, 53.0, 1014.0),
            (1713.0, 51.0, 1017.0),
            (1710.0, 49.0, 1021.0),
            (1707.0, 47.0, 1024.0),
            (1706.0, 45.0, 1028.0),
            (1709.0, 44.0, 1030.0),
            (1721.0, 32.0, 949.0),
            (1727.0, 31.0, 949.0),
            (1731.0, 30.0, 947.0),
            (1734.0, 29.0, 945.0),
            (1738.0, 28.0, 946.0),
            (1743.0, 27.0, 948.0),
            (1746.0, 26.0, 952.0),
            (1749.0, 25.0, 956.0),
            (1752.0, 24.0, 959.0),
            (1756.0, 23.0, 963.0),
            (1760.0, 22.0, 966.0),
            (1764.0, 21.0, 970.0),
            (1768.0, 20.0, 972.0),
            (1772.0, 19.0, 975.0),
            (1748.0, 13.0, 1021.0),
            (1744.0, 12.0, 1024.0),
            (1739.0, 11.0, 1028.0),
            (1734.0, 11.0, 1029.0),
            (1729.0, 11.0, 1030.0),
            (1724.0, 11.0, 1032.0),
            (1719.0, 10.0, 1035.0),
            (1714.0, 9.0, 1039.0),
            (1691.0, 8.0, 1056.0),
            (1688.0, 8.0, 1059.0),
            (1698.0, 16.0, 1105.0),
            (1700.0, 18.0, 1109.0),
            (1704.0, 19.0, 1112.0),
            (1707.0, 20.0, 1116.0),
            (1710.0, 21.0, 1120.0),
            (1713.0, 22.0, 1124.0),
            (1717.0, 23.0, 1126.0),
            (1722.0, 24.0, 1126.0),
            (1726.0, 25.0, 1124.0),
            (1729.0, 26.0, 1122.0),
            (1732.0, 27.0, 1124.0),
            (1787.0, 34.0, 1139.0),
            (1791.0, 35.0, 1138.0),
            (1794.0, 36.0, 1135.0),
            (1797.0, 37.0, 1132.0),
            (1798.0, 38.0, 1128.0),
            (1800.0, 39.0, 1124.0),
            (1804.0, 39.0, 1120.0),
            (1808.0, 39.0, 1117.0),
            (1813.0, 39.0, 1117.0),
            (1818.0, 40.0, 1118.0),
            (1823.0, 40.0, 1118.0),
            (1828.0, 40.0, 1118.0),
            (1833.0, 40.0, 1117.0),
            (1838.0, 39.0, 1116.0),
            (1842.0, 38.0, 1118.0),
            (1847.0, 37.0, 1120.0),
            (1851.0, 36.0, 1123.0),
            (1855.0, 36.0, 1126.0),
            (1858.0, 36.0, 1130.0),
            (1860.0, 35.0, 1135.0),
            (1860.0, 35.0, 1140.0),
            (1859.0, 35.0, 1145.0),
            (1855.0, 35.0, 1149.0),
            (1851.0, 35.0, 1153.0),
            (1848.0, 36.0, 1156.0),
            (1844.0, 36.0, 1159.0),
            (1840.0, 36.0, 1162.0),
            (1837.0, 37.0, 1165.0),
            (1833.0, 37.0, 1166.0),
            (1829.0, 38.0, 1166.0),
            (1825.0, 39.0, 1165.0),
            (1821.0, 40.0, 1164.0),
            (1817.0, 41.0, 1164.0),
            (1813.0, 42.0, 1162.0),
            (1809.0, 43.0, 1161.0),
            (1806.0, 44.0, 1159.0),
            (1803.0, 45.0, 1157.0),
            (1800.0, 46.0, 1154.0),
            (1800.0, 47.0, 1151.0),
            (1798.0, 48.0, 1148.0),
            (1797.0, 49.0, 1145.0),
            (1797.0, 50.0, 1142.0),
            (1798.0, 51.0, 1139.0),
            (1800.0, 52.0, 1136.0),
            (1802.0, 53.0, 1133.0),
            (1805.0, 54.0, 1130.0),
            (1792.0, 18.0, 986.0),
            (1795.0, 19.0, 983.0),
            (1799.0, 19.0, 979.0),
            (1804.0, 19.0, 976.0),
        ];
        for coord in coords {
            let cloud = clouds[rand::thread_rng().gen_range(0..clouds.len())].clone();
            // TODO: can we instantiate ItemDisplay just once?
            let display = ItemDisplay::new(&cloud)?;
            display.set_no_gravity(true)?;
            let yaw = rand::thread_rng().gen_range(0..4) as f32 * 90.0;
            let yaw_variation = rand::thread_rng().gen_range(-15..=15) as f32;
            let pitch = rand::thread_rng().gen_range(-5..=5) as f32;
            display.spawn(
                &self.instance,
                coord.0,
                coord.1,
                coord.2,
                yaw + yaw_variation,
                pitch,
            )?;
        }

        Ok(())
    }

    fn instance(&self) -> Arc<InstanceContainer> {
        self.instance.clone()
    }
}
