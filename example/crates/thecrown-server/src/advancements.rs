use crate::commands::SpawnCommand;
use crate::logic::lobby::LobbyServer;
use minestom::advancement::FrameType;
use crate::logic::parkour::ParkourServer;
use crate::magic_values::*;
use minestom::advancement::Advancement;
use minestom::advancement::AdvancementRoot;
use minestom::advancement::AdvancementManager;
use crate::maps::LobbyMap2;
use crate::maps::map::LobbyMap;
use crate::mojang::get_skin_and_signature;
use log::{error, info};
use minestom;
use minestom::InstanceContainer;
use minestom::MinestomServer;
use minestom::ServerListPingEvent;
use minestom::TOKIO_HANDLE;
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
use minestom::Player;
use std::path::Path;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::{Mutex, RwLock};
use uuid::Uuid;
use world_seed_entity_engine::model_engine::ModelEngine;

pub const WELCOME: &'static str = "welcome";
pub const SHRUNKEN: &'static str = "shrunken";
pub const TITANOMACHY: &'static str = "titanomachy";

impl AdvancementsMap {
    pub fn new(minecraft_server: &MinestomServer, player: &Player) -> minestom::Result<Self> {
        let adv_manager = minecraft_server.advancement_manager()?;

        let map: HashMap<String, Advancement> = crate::define_advancements! {
            server: minecraft_server,
            player: player,
            tabs: [
                {
                    tab_name: "thecrown",
                    items: [
                        {
                            name: WELCOME,
                            title: "Welcome to TheCrown!",
                            description: "You can now start playing.",
                            icon: Material::BlueBed,
                            frame: FrameType::GOAL(),
                            coords: (0.0, 0.0),
                            background: "thecrown:textures/block/zanite_block.png",
                            achieved: true // Default achievement
                        },
                        {
                            name: SHRUNKEN,
                            title: "Honey, I Shrunk Myself!",
                            description: "Squeezed through where no Steve has gone before... and found a tiny secret worth the squeeze - Find the tiny hidden hole only the smallest can fit through.",
                            icon: Material::GoldIngot,
                            frame: FrameType::CHALLENGE(),
                            coords: (1.0, 2.0),
                            depends_on: WELCOME,
                            achieved: false
                        },
                        {
                            name: TITANOMACHY,
                            title: "Honey, I Shrunk Myself!",
                            description: "The mighty were cast down, and Tartarus awaited - Step off the edge into the void below, and embrace your destiny as a fallen giant.",
                            icon: Material::GoldIngot,
                            frame: FrameType::CHALLENGE(),
                            coords: (1.0, 2.0),
                            depends_on: WELCOME,
                            achieved: false
                        }
                    ]
                },
                {
                    tab_name: "bonus_tab",
                    items: [
                        {
                            name: "bonus_root",
                            title: "Bonus Root",
                            description: "This is a standalone bonus root",
                            icon: Material::Emerald,
                            frame: FrameType::CHALLENGE(),
                            coords: (0.0, 0.0),
                            background: "minecraft:textures/block/gold_block.png",
                            achieved: true
                        },
                        {
                            name: "bonus_child",
                            title: "Bonus Child",
                            description: "Follows the bonus_root advancement",
                            icon: Material::Diamond,
                            frame: FrameType::TASK(),
                            coords: (2.0, -1.0),
                            depends_on: "bonus_root",
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

pub fn init_player_advancements(minecraft_server: &MinestomServer, player: &Player) -> minestom::Result<()> {
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
}

impl CanAchieveAdvancement for Player {
    fn set_achieved(&self, name: &str) -> minestom::Result<()> {
        get_advancement(&self, name).unwrap().set_achieved(true)?;
        Ok(())
    }
}

static ADVANCEMENTS: LazyLock<RwLock<HashMap<Uuid, AdvancementsMap>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

// Place this macro in a crate‐wide scope (e.g. in `lib.rs` or a top‐level module marked with `#[macro_use]`),
// so that you can invoke it as `crate::define_advancements! { … }`.

#[macro_export]
macro_rules! define_advancements {
    (
        server: $server:expr,
        player: $player:expr,
        tabs: [
            $(
                {
                    tab_name: $tab_name:expr,
                    items: [
                        // The first entry in each `items` array is the root for that tab.
                        {
                            name: $root_name:expr,
                            title: $root_title:expr,
                            description: $root_desc:expr,
                            icon: $root_icon:expr,
                            frame: $root_frame:expr,
                            coords: ($root_x:expr, $root_y:expr),
                            background: $root_bg:expr,
                            achieved: $root_ach:expr
                        }
                        $(
                            // All subsequent entries must specify a `depends_on` pointing
                            // to a previously‐listed name in the same tab.
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

        // 2) Build a single HashMap<String, Advancement> to collect every root & child.
        let mut adv_map: std::collections::HashMap<String, minestom::advancement::Advancement> =
            std::collections::HashMap::new();

        $(
            // ───────────────────────────────────────────────────────
            // For each `{ tab_name, items: [ … ] }` block:
            //   1. Create the root AdvancementRoot (first `items` entry).
            //   2. Create exactly one tab named `$tab_name`.
            //   3. Put `root.as_advancement()` into adv_map under `$root_name`.
            //   4. Iterate over the remaining child entries, creating each under its parent.
            // ───────────────────────────────────────────────────────
            {
                // 1) Build the root using `AdvancementRoot::new(...)`, which now always requires `background: &str`.
                let root = minestom::advancement::AdvancementRoot::new(
                    &component!($root_title),
                    &component!($root_desc),
                    $root_icon,
                    $root_frame,
                    $root_x,
                    $root_y,
                    $root_bg,   // Always a &str, never optional
                )?;
                // Optionally mark the root as already achieved:
                if $root_ach {
                    let _ = root.as_advancement().set_achieved(true);
                }

                // 2) Create a single AdvancementTab named `$tab_name`:
                let tab = adv_manager.create_tab($tab_name, root.clone())?;
                tab.add_viewer($player)?;

                // 3) Insert the root’s Advancement (root.as_advancement()) into adv_map
                adv_map.insert($root_name.to_string(), root.as_advancement());

                // 4) Process each “child” entry under the same tab:
                $(
                    // a) Look up the parent’s Advancement (it must already exist in adv_map).
                    let parent_adv: minestom::advancement::Advancement =
                        adv_map.get($parent).unwrap().clone();

                    // b) Build a new plain Advancement (no background for children).
                    let adv = minestom::advancement::Advancement::new(
                        &component!($title),
                        &component!($desc),
                        $icon,
                        $frame,
                        $x,
                        $y,
                    )?;
                    let _ = adv.show_toast(true);

                    // c) Give it an ID that uses the tab name as a namespace:
                    let id = format!("{}/{}", $tab_name, $name);
                    tab.create_advancement(&id, adv.clone(), parent_adv)?;

                    // d) If `achieved: true`, mark it immediately
                    if $ach {
                        let _ = adv.set_achieved(true);
                    }

                    // e) Insert this child into adv_map so deeper descendants can refer to it.
                    adv_map.insert($name.to_string(), adv);
                )*
            }
        )*

        // Return one combined HashMap of “name → Advancement” across all tabs.
        adv_map
    }};
}
