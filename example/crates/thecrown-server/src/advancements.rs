use crate::commands::SpawnCommand;
use crate::logic::lobby::LobbyServer;
use crate::logic::parkour::ParkourServer;
use crate::magic_values::*;
use crate::maps::LobbyMap2;
use crate::maps::map::LobbyMap;
use crate::mojang::get_skin_and_signature;
use log::{error, info};
use minestom;
use minestom::InstanceContainer;
use minestom::MinestomServer;
use minestom::Player;
use minestom::ServerListPingEvent;
use minestom::TOKIO_HANDLE;
use minestom::advancement::Advancement;
use minestom::advancement::AdvancementManager;
use minestom::advancement::AdvancementRoot;
use minestom::advancement::FrameType;
use minestom::entity::PlayerSkin;
use minestom::{
    attribute::Attribute,
    command::{Command, CommandContext},
    component,
    entity::GameMode,
    event::player::{
        AsyncPlayerConfigurationEvent, PlayerMoveEvent, PlayerSkinInitEvent, PlayerSpawnEvent,
    },
    item::{InventoryHolder, ItemStack},
    material::Material,
    resource_pack::{ResourcePackInfo, ResourcePackRequest, ResourcePackRequestBuilder},
};
use parking_lot::Mutex as ParkingMutex;
use rand::Rng;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::{Mutex, RwLock};
use uuid::Uuid;
use world_seed_entity_engine::model_engine::ModelEngine;

pub const WELCOME: &'static str = "welcome";
pub const SHRUNKEN: &'static str = "shrunken";
pub const TITANOMACHY: &'static str = "titanomachy";

pub const PARKOUR_WELCOME: &'static str = "parkour_welcome";

// TODO: uncache advancements when player leaves

impl AdvancementsMap {
    pub fn new(minecraft_server: &MinestomServer, player: &Player) -> minestom::Result<Self> {
        let adv_manager = minecraft_server.advancement_manager()?;
        let prefix = player.get_username()?.to_lowercase();

        // TODO: remove when we can uncache advancements
        let suffix: String = (0..5)
            .map(|_| rand::thread_rng().gen_range('a'..='z'))
            .collect();
        let prefix = format!("{}-{}", prefix, suffix);

        let map: HashMap<String, Advancement> = crate::define_advancements! {
            server: minecraft_server,
            player: player,
            tabs: [
                {
                    tab_name: &format!("{}-network", prefix),
                    background: "minecraft:textures/block/stripped_acacia_log.png",
                    items: [
                        {
                            name: WELCOME,
                            title: component!("Welcome to TheCrown!"),
                            description: component!("You can now start playing."),
                            icon: Material::BlueBed,
                            frame: FrameType::GOAL(),
                            coords: (0.0, 0.0),
                            achieved: true
                        },
                        {
                            name: SHRUNKEN,
                            title: component!("Honey, I Shrunk Myself!").bold(),
                            description: component!("You squeezed through where no Steve has gone before... and found a tiny secret worth the squeeze")
                                .color("#adadad")?
                                .chain(component!(" - ").color("#FFFFFF")?)
                                .chain(component!("Find the tiny hidden hole only the smallest can fit through.").color("#454545")?),
                            icon: Material::JungleButton,
                            frame: FrameType::CHALLENGE(),
                            coords: (2.0, 1.0),
                            depends_on: WELCOME,
                            achieved: false
                        },
                        {
                            name: TITANOMACHY,
                            title: component!("Titanomachy").bold(),
                            description: component!("The mighty were cast down, and Tartarus awaited")
                                .color("#adadad")?
                                .chain(component!(" - ").color("#FFFFFF")?)
                                .chain(component!("Step off the edge into the void below.").color("#454545")?),
                            icon: Material::IronBlock,
                            frame: FrameType::CHALLENGE(),
                            coords: (2.0, -1.0),
                            depends_on: WELCOME,
                            achieved: false
                        }
                    ]
                },
                {
                    tab_name: &format!("{}-parkour", prefix),
                    background: "minecraft:textures/block/stripped_warped_stem.png",
                    items: [
                        {
                            name: PARKOUR_WELCOME,
                            title: component!("Welcome to parkour!"),
                            description: component!("The first 25, and many more to go!"),
                            icon: Material::Dirt,
                            frame: FrameType::GOAL(),
                            coords: (0.0, 0.0),
                            achieved: false
                        }
                    ]
                }
            ]
        };

        Ok(Self { advancements: map })
    }
}

pub struct AdvancementsMap {
    pub advancements: HashMap<String, Advancement>,
}

pub fn init_player_advancements(
    minecraft_server: &MinestomServer,
    player: &Player,
) -> minestom::Result<()> {
    let uuid = player.get_uuid()?;
    let adv_map = AdvancementsMap::new(minecraft_server, player)?;
    let mut global = ADVANCEMENTS.write().unwrap();
    global.insert(uuid, adv_map);
    Ok(())
}

pub fn get_advancement(player: &Player, name: &str) -> Option<Advancement> {
    let advancements = ADVANCEMENTS.read().unwrap();
    if let Some(player_advancements) = advancements.get(&player.get_uuid().ok()?) {
        player_advancements.advancements.get(name).cloned()
    } else {
        None
    }
}

pub trait CanAchieveAdvancement {
    fn set_achieved(&self, name: &str) -> minestom::Result<()>;
    fn is_achieved(&self, name: &str) -> minestom::Result<bool>;
}

impl CanAchieveAdvancement for Player {
    fn set_achieved(&self, name: &str) -> minestom::Result<()> {
        let adv = get_advancement(&self, name).unwrap();
        adv.set_achieved(true)?;
        adv.show_toast(false)?;
        Ok(())
    }

    fn is_achieved(&self, name: &str) -> minestom::Result<bool> {
        get_advancement(&self, name).unwrap().is_achieved()
    }
}

static ADVANCEMENTS: LazyLock<RwLock<HashMap<Uuid, AdvancementsMap>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

#[macro_export]
macro_rules! define_advancements {
    (
        server: $server:expr,
        player: $player:expr,
        tabs: [
            $(
                {
                    tab_name: $tab_name:expr,
                    background: $tab_bg:expr,
                    items: [
                        // (A) Optional “root” entry: this must supply `title: <Component>` and `description: <Component>`.
                        $(
                            {
                                name: $root_name:expr,
                                title: $root_title:expr,
                                description: $root_desc:expr,
                                icon: $root_icon:expr,
                                frame: $root_frame:expr,
                                coords: ($root_x:expr, $root_y:expr),
                                achieved: $root_ach:expr
                            }
                        )?

                        // (B) Zero or more “child” entries. Each child must supply `depends_on: <&str>`.
                        $(
                            ,
                            {
                                name: $name:expr,
                                title: $title:expr,
                                description: $desc:expr,
                                icon: $icon:expr,
                                frame: $frame:expr,
                                coords: ($x:expr, $y:expr),
                                depends_on: $parent:expr,
                                achieved: $ach:expr
                            }
                        )*
                    ]
                }
            ),* $(,)?
        ]
    ) => {{
        // 1) Grab the AdvancementManager once for all tabs.
        let adv_manager = $server.advancement_manager()?;

        // 2) Build one HashMap<String, Advancement> to collect everything.
        let mut adv_map: std::collections::HashMap<String, minestom::advancement::Advancement> =
            std::collections::HashMap::new();

        $(
            {
                // (A) Build a root if the optional root‐entry is present:
                $(
                    let root = minestom::advancement::AdvancementRoot::new(
                        &$root_title,   // must be a component!(…)
                        &$root_desc,    // must be a component!(…)
                        $root_icon,
                        $root_frame,
                        $root_x,
                        $root_y,
                        $tab_bg,        // &str
                    )?;
                    let root_adv = root.as_advancement();
                    if $root_ach {
                        let _ = root_adv.set_achieved(true);
                    }
                    // Show toast if not yet achieved.
                    let _ = root_adv.show_toast(!$root_ach);
                    let tab = adv_manager.create_tab($tab_name, root.clone())?;
                    tab.add_viewer($player)?;
                    adv_map.insert($root_name.to_string(), root_adv);
                )?

                // (B) Process zero or more child entries:
                $(
                    let parent_adv: minestom::advancement::Advancement =
                        adv_map.get($parent).unwrap().clone();
                    let adv = minestom::advancement::Advancement::new(
                        &$title,      // component!(…)
                        &$desc,       // component!(…)
                        $icon,
                        $frame,
                        $x,
                        $y,
                    )?;

                    let id = format!("{}/{}", $tab_name, $name);
                    tab.create_advancement(&id, adv.clone(), parent_adv)?;
                    if $ach {
                        let _ = adv.set_achieved(true);
                    }
                    // Show toast if not yet achieved.
                    let _ = adv.show_toast(!$ach);
                    adv_map.insert($name.to_string(), adv);
                )*
            }
        )*

        adv_map
    }};
}
