use minestom_rs::Result;
use std::sync::Arc;
use minestom_rs::PlayerEntityInteractEvent;
use minestom_rs::PlayerMoveEvent;
use minestom_rs::entity::display::ItemDisplay;
use minestom_rs::entity::entity::Entity;
use minestom_rs::item::ItemStack;
use minestom_rs::entity::entity::EntityType;
use minestom_rs::material::Material;
use minestom_rs::instance::InstanceContainer;
use minestom_rs::collision::BoundingBox;

pub fn spawn_piano(instance: Arc<InstanceContainer>, x: f64, y: f64, z: f64, length: f64, yaw: f32) -> Result<()> {
    // The spawn point is 1/4 from the left.
    // The coordinates provided are of the left point
    let piano = ItemStack::of(Material::Diamond)?
        .with_amount(1)?
        .with_custom_model_data("piano")?;
    
    let base_length = 2.0;
    let scale = length / base_length;
    let sin = (yaw as f64).to_radians().sin();
    let cos = (yaw as f64).to_radians().cos();
    let pitch = 0.0; // We only support yaw
    let x_center = x - cos * length / 2.0;
    let y_center = y;
    let z_center = z - sin * length / 2.0;
    let interaction_radius = 3.0 * scale;
    
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

    // Spawn an invisible armor stand at the same location
    let armor_stand = Entity::new_from_type(EntityType::ArmorStand)?;
    armor_stand.set_invisible(false)?;
    armor_stand.set_no_gravity(true)?;
    armor_stand.set_bounding_box(&BoundingBox::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)?)?;
    armor_stand.spawn(
        &instance,
        1774.5,
        30.0,
        1056.7,
        45.0,
        90.0,
    )?;

    let event_node = instance.event_node()?;

    event_node.listen(move |event: &PlayerEntityInteractEvent| {
        // print interact location
        let pos = event.get_interact_position()?;
        let r#type = armor_stand.get_type()?;
        log::info!("Interact location {:?} : {} {} {}", r#type, pos.x, pos.y, pos.z);
        Ok(())
    })?;

    Ok(())
}