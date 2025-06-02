use minestom::Player;
use minestom::PlayerEntityInteractEvent;
use minestom::PlayerMoveEvent;
use minestom::Result;
use minestom::Source;
use minestom::collision::BoundingBox;
use minestom::entity::display::ItemDisplay;
use minestom::entity::entity::Entity;
use minestom::entity::entity::EntityType;
use minestom::instance::InstanceContainer;
use minestom::item::ItemStack;
use minestom::material::Material;
use minestom::particle::ParticlePacket;
use minestom::particle::ParticleType;
use minestom::sound::Sound;
use minestom::sound::SoundEvent;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

// ID so that multiple pianos can be spawned in the same instance
static COUNTER: Lazy<RwLock<u32>> = Lazy::new(|| RwLock::new(0));

fn read_and_increment_counter() -> u32 {
    let mut counter = COUNTER.write();
    let current = *counter;
    *counter += 1;
    current
}

pub fn spawn_piano(
    instance: InstanceContainer,
    players: Arc<RwLock<HashMap<Uuid, Player>>>,
    x: f64,
    y: f64,
    z: f64,
    yaw: f32,
) -> Result<()> {
    let id = read_and_increment_counter();
    let length = 3.0;
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
    // TODO: this scaling does not work well for length>3
    // Direction towards the "playing player"
    // x += sin and z -= cos
    let offset1 = scale * 0.05;
    // Direction from left to right of the piano
    // x -= cos and z -= sin
    let offset2 = scale * 0.34;
    let offsetY = scale * (1.0 / 16.0) * 4.5;

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

    let tag_id = format!("tiles_section_{}", id);

    // fill the area with armostands to cover all the tiles,
    // sinc we cannot have a large hitbox
    for i in 0..(tiles_length / armor_width).ceil() as i32 {
        let armor_stand = Entity::new_from_type(EntityType::ArmorStand)?;
        armor_stand.set_invisible(true)?;
        armor_stand.set_no_gravity(true)?;
        armor_stand.spawn(
            &instance,
            x + (sin * offset1) - (cos * offset2) - (cos * armor_width * i as f64),
            y - 1.0 + offsetY,
            z - (cos * offset1) - (sin * offset2) - (sin * armor_width * i as f64),
            yaw,
            pitch,
        )?;
        let tag_handler = armor_stand.tag_handler()?;
        tag_handler.set_tag(&tag_id, Some(&i.to_string()))?;
    }

    let event_node = instance.event_node()?;

    event_node.listen(move |event: &PlayerEntityInteractEvent| {
        let pos = event.get_interact_position()?;
        let player = event.get_player()?;
        if pos.y > max_depth_interaction {
            let entity = event.get_target()?;
            let tag_handler = entity.tag_handler()?;
            if let Some(value) = tag_handler.get_tag(&tag_id)? {
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
                    if normalized_tile_coordinate < 1.1 { // ideally 1.0 but it's not that precise
                        // There are 15 tiles of alternating width such as abababa
                        // with the following sizes
                        let tiles = 15;
                        let a = 0.051333333 * scale;
                        let b = 0.042666666 * scale;
                        let res = find_tile_index(normalized_tile_coordinate, tiles, a, b);

                        if let Some((tile_index, tile_middle_point)) = res {
                            let avg = (a + b) * 0.5; // it doesn't need to be precise.
                            // where to spawn the note + sound source (middle point of the tile)
                            // we need to de-normalized
                            let denormalized_middle_point = tile_middle_point * tiles_length;
                            let note_offset =
                                offset2 - armor_width * 0.5 + denormalized_middle_point;
                            let offset1 = scale * 0.1; // closer to the player
                            let source_x = x + (sin * offset1) - (cos * note_offset);
                            let source_y = y - offsetY + 2.0;
                            let source_z = z - (cos * offset1) - (sin * note_offset);

                            play_tile(
                                players.clone(),
                                tile_index,
                                tiles,
                                source_x,
                                source_y,
                                source_z,
                            )?;
                        }
                    }
                }
            }
        }
        Ok(())
    })?;

    Ok(())
}

fn play_tile(
    players: Arc<RwLock<HashMap<Uuid, Player>>>,
    tile_index: usize,
    tiles: usize,
    x: f64,
    y: f64,
    z: f64,
) -> Result<()> {
    // create particle packet
    let mut particle_packet = ParticlePacket::new(ParticleType::Note, x, y, z);

    // pitch is in [0.5, 2]
    // for every player
    let normalized_pitch = (tile_index as f32) / ((tiles - 1) as f32);
    let pitch = normalized_pitch * 1.5 + 0.5;
    for player in players.read().values() {
        player.play_sound_at(
            &Sound::sound(SoundEvent::BlockNoteBlockBass, Source::Record, 1.0, pitch)?,
            x,
            y,
            z,
        )?;

        player.send_packet(&particle_packet)?;
    }
    Ok(())
}

/// Returns the tile index and the middle point of the title
fn find_tile_index(x: f64, tiles: usize, a: f64, b: f64) -> Option<(usize, f64)> {
    let mut pos = 0.0;

    for i in 0..tiles {
        let length = if i % 2 == 0 { a } else { b };
        if x >= pos && x <= pos + length {
            // Check is x is in the current tile
            return Some((i, (pos + length * 0.5)));
        }
        pos += length;
    }

    // If x is beyond the last tile, return None
    None
}
