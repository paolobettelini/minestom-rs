use crate::collision::BoundingBox;
use crate::Result;
use crate::item::ItemStack;
use crate::jni_utils::{JavaObject, JniValue, get_env};
use crate::InstanceContainer;
use jni::objects::JValue;

pub struct ItemDisplay {
    inner: JavaObject,
}

impl ItemDisplay {
    /// Creates a new ItemDisplay with the given item
    pub fn new(item: &ItemStack) -> Result<Self> {
        let mut env = get_env()?;
        
        // Get the EntityType for ItemDisplay
        let entity_type_class = env.find_class("net/minestom/server/entity/EntityType")?;
        let entity_type = env.get_static_field(
            entity_type_class,
            "ITEM_DISPLAY",
            "Lnet/minestom/server/entity/EntityType;"
        )?;
        
        // Create a new Entity with ITEM_DISPLAY type
        let entity_class = env.find_class("net/minestom/server/entity/Entity")?;
        let entity_type_obj = entity_type.l()?;
        let entity = env.new_object(
            entity_class,
            "(Lnet/minestom/server/entity/EntityType;)V",
            &[JValue::Object(&entity_type_obj)],
        )?;

        // Get the ItemDisplayMeta
        let meta = env.call_method(
            &entity,
            "getEntityMeta",
            "()Lnet/minestom/server/entity/metadata/EntityMeta;",
            &[],
        )?;

        // Set the item stack on the meta
        env.call_method(
            meta.l()?,
            "setItemStack",
            "(Lnet/minestom/server/item/ItemStack;)V",
            &[JValue::Object(&item.as_obj().as_obj()?)],
        )?;

        Ok(Self {
            inner: JavaObject::from_env(&mut env, entity)?,
        })
    }

    /// Sets the instance and position of this ItemDisplay in one call
    pub fn spawn(&self, instance: &InstanceContainer, x: f64, y: f64, z: f64, yaw: f32, pitch: f32) -> Result<()> {
        let mut env = get_env()?;
        
        // Create Pos object
        let pos_class = env.find_class("net/minestom/server/coordinate/Pos")?;
        let pos = env.new_object(
            pos_class,
            "(DDDFF)V",
            &[
                JValue::Double(x),
                JValue::Double(y),
                JValue::Double(z),
                JValue::Float(yaw),
                JValue::Float(pitch),
            ],
        )?;

        // Call setInstance with position which returns a CompletableFuture
        let future = self.inner.call_object_method(
            "setInstance",
            "(Lnet/minestom/server/instance/Instance;Lnet/minestom/server/coordinate/Pos;)Ljava/util/concurrent/CompletableFuture;",
            &[
                JniValue::Object(instance.inner()?),
                JniValue::Object(pos),
            ],
        )?;

        // Wait for the operation to complete
        env.call_method(
            future.as_obj()?,
            "join",
            "()Ljava/lang/Object;",
            &[],
        )?;

        Ok(())
    }

    pub fn set_bounding_box(&self, box_: &BoundingBox) -> Result<()> {
        let mut env = get_env()?;
        let entity_obj = self.inner.as_obj()?;
        env.call_method(
            entity_obj,
            "setBoundingBox",
            "(Lnet/minestom/server/collision/BoundingBox;)V",
            &[JValue::Object(&box_.as_java().as_obj()?)]
        )?;
        Ok(())
    }

    /// Sets whether this ItemDisplay should be affected by gravity
    pub fn set_no_gravity(&self, no_gravity: bool) -> Result<()> {
        let mut env = get_env()?;

        // Ottieni il metadata dell'entità
        let meta_obj = self.inner.call_object_method(
            "getEntityMeta",
            "()Lnet/minestom/server/entity/metadata/EntityMeta;",
            &[],
        )?;

        // Chiama setHasNoGravity(boolean)
        env.call_method(
            &meta_obj.as_obj()?,
            "setHasNoGravity",
            "(Z)V",
            &[JValue::Bool(if no_gravity { 1 } else { 0 })],
        )?;

        Ok(())
    }

    /// Sets the scale of this ItemDisplay
    pub fn set_scale(&self, x: f32, y: f32, z: f32) -> Result<()> {
        let mut env = get_env()?;
        
        // Create Vec object
        let vec_class = env.find_class("net/minestom/server/coordinate/Vec")?;
        let vec_obj = env.new_object(
            vec_class,
            "(DDD)V",
            &[
                JValue::Double(x as f64),
                JValue::Double(y as f64),
                JValue::Double(z as f64),
            ],
        )?;

        // Get the metadata
        let meta_obj = self.inner.call_object_method(
            "getEntityMeta",
            "()Lnet/minestom/server/entity/metadata/EntityMeta;",
            &[],
        )?;

        // Set the scale
        env.call_method(
            &meta_obj.as_obj()?,
            "setScale",
            "(Lnet/minestom/server/coordinate/Vec;)V",
            &[JValue::Object(&vec_obj)],
        )?;

        Ok(())
    }

    /// Sets the brightness of this ItemDisplay
    pub fn set_brightness(&self, block_light: i32, sky_light: i32) -> Result<()> {
        let meta_obj = self.inner.call_object_method(
            "getEntityMeta",
            "()Lnet/minestom/server/entity/metadata/EntityMeta;",
            &[],
        )?;

        let mut env = get_env()?;
        env.call_method(
            &meta_obj.as_obj()?,
            "setBrightness",
            "(II)V",
            &[
                JValue::Int(block_light),
                JValue::Int(sky_light),
            ],
        )?;

        Ok(())
    }

    /// Sets whether this ItemDisplay should be visible
    pub fn set_invisible(&self, invisible: bool) -> Result<()> {
        let meta_obj = self.inner.call_object_method(
            "getEntityMeta",
            "()Lnet/minestom/server/entity/metadata/EntityMeta;",
            &[],
        )?;

        let mut env = get_env()?;
        env.call_method(
            &meta_obj.as_obj()?,
            "setInvisible",
            "(Z)V",
            &[JValue::Bool(if invisible { 1 } else { 0 })],
        )?;

        Ok(())
    }
} 