use minestom_rs::PlayerEntityInteractEvent;
use minestom_rs::PlayerMoveEvent;
use minestom_rs::Result;
use minestom_rs::collision::BoundingBox;
use minestom_rs::entity::display::ItemDisplay;
use minestom_rs::entity::entity::Entity;
use minestom_rs::entity::entity::EntityType;
use minestom_rs::instance::InstanceContainer;
use minestom_rs::Source;
use minestom_rs::sound::Sound;
use minestom_rs::sound::SoundEvent;
use minestom_rs::item::ItemStack;
use minestom_rs::material::Material;
use std::sync::Arc;
use uuid::Uuid;

pub fn spawn_piano(
    instance: Arc<InstanceContainer>,
    x: f64,
    y: f64,
    z: f64,
    length: f64,
    yaw: f32,
) -> Result<()> {
    // The spawn point is 1/4 from the left.
    // The coordinates provided are of the left point
    let piano = ItemStack::of(Material::Diamond)?
        .with_amount(1)?
        .with_custom_model_data("piano")?;

    let base_length = 2.0;
    let scale = length / base_length;
    let sin = (yaw as f64).to_radians().sin();
    let cos = (yaw as f64).to_radians().cos();
    let pitch = 0.0; // We only support yaw (multiple of 90)
    let x_center = x - cos * length / 2.0;
    let y_center = y;
    let z_center = z - sin * length / 2.0;
    let interaction_radius = 3.0 * scale;
    let armor_width = 0.5;
    let tiles_length = 1.533333 * scale;
    let max_depth_interaction = 1.9; // height on the armostand to consider it a click

    let display = ItemDisplay::new(&piano)?;
    display.set_no_gravity(true)?;
    display.set_scale(scale as f32, scale as f32, scale as f32)?;
    display.spawn(
        &instance,
        x - cos * length / 4.0,
        y + 0.5 * scale,
        z - sin * length / 4.0,
        yaw,
        pitch,
    )?;

    // fill the area with armostands to cover all the tiles,
    // sinc we cannot have a large hitbox
    for i in 0..(tiles_length / armor_width).ceil() as i32 {
        let armor_stand = Entity::new_from_type(EntityType::ArmorStand)?;
        armor_stand.set_invisible(false)?;
        armor_stand.set_no_gravity(true)?;
        armor_stand.set_bounding_box(&BoundingBox::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)?)?;
        // TODO: this scaling does not work well for length>3
        // Direction towards the "playing player"
        // x += sin and z -= cos
        let offset1 = scale * 0.05;
        // Direction from left to right of the piano
        // x -= cos and z -= sin
        let offset2 = scale * 0.35;
        let offsetY = scale * (1.0 / 16.0) * 4.5;
        armor_stand.spawn(
            &instance,
            x + (sin * offset1) - (cos * offset2) - (cos * armor_width * i as f64),
            y - 1.0 + offsetY,
            z - (cos * offset1) - (sin * offset2) - (sin * armor_width * i as f64),
            yaw,
            pitch,
        )?;
        let tag_handler = armor_stand.tag_handler()?;
        tag_handler.set_tag("tiles_section", Some(&i.to_string()))?;
    }

    let event_node = instance.event_node()?;

    event_node.listen(move |event: &PlayerEntityInteractEvent| {
        let pos = event.get_interact_position()?;
        let player = event.get_player()?;
        if pos.y > max_depth_interaction {
            let entity = event.get_target()?;
            let tag_handler = entity.tag_handler()?;
            if let Some(value) = tag_handler.get_tag("tiles_section")? {
                if let Ok(armorstand_index) = value.parse::<i32>() {
                    // From the segmented hitboxes we need to construct
                    // the relative coordinate of the full tiles.
                    let previous_sections = (armorstand_index as f64) * armor_width;
                    // coordinate from left to right.
                    let relative_section_coordinate = -cos * pos.x - sin * pos.z;
                    // go from [-width/2, width/2] to [0, width]
                    let abs_section_coordinate = relative_section_coordinate + armor_width * 0.5;
                    let tile_coordinate = previous_sections + abs_section_coordinate;
                    // now we need to normalize this value since the piano is scaled
                    let normalized_tile_coordinate = tile_coordinate / tiles_length;
                    // This goes from 0 to 1+ the extra depends on how well
                    // the last armorstand fits but we can just discard it.
                    if (normalized_tile_coordinate < tile_coordinate) {
                        // There are 15 tiles of alternating width such as abababa
                        // with the following sizes
                        let tiles = 15;
                        let a = 0.077;
                        let b = 0.064;
                        let index = find_tile_index(normalized_tile_coordinate, tiles, a, b);
                        if let Some(tile_index) = index {
                            // pitch is in [0.5, 2]
                            let pitch = (tile_index as f32) / ((tiles - 1)as f32) * 1.5 + 0.5;
                            player.play_sound(&Sound::sound(
                                SoundEvent::BlockNoteBlockBass,
                                Source::Record,
                                1.0,
                                pitch,
                            )?)?;
                        }
                    }
                }
            }
        }
        Ok(())
    })?;

    Ok(())
}

fn find_tile_index(x: f64, tiles: usize, a: f64, b: f64) -> Option<usize> {
    let mut pos = 0.0;

    for i in 0..tiles {
        let length = if i % 2 == 0 { a } else { b };
        if x >= pos && x <= pos + length { // Check is x is in the current tile
            return Some(i);
        }
        pos += length;
    }

    // If x is beyond the last tile, return None
    None
}
