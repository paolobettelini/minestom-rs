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
        let entity_type_class = env.find_class("net/minestom/server/entity/EntityTypes")?;
        let entity_type = env.get_static_field(
            entity_type_class,
            "ITEM_DISPLAY",
            "Lnet/minestom/server/entity/EntityType;"
        )?;
        let entity_type_obj = entity_type.l()?;

        // Create the Entity instance
        let display_class = env.find_class("net/minestom/server/entity/Entity")?;
        let display_obj = env.new_object(
            display_class,
            "(Lnet/minestom/server/entity/EntityType;)V",
            &[JValue::Object(&entity_type_obj)],
        )?;

        // Get the MetadataHolder class
        let metadata_holder_class = env.find_class("net/minestom/server/entity/MetadataHolder")?;

        // Create MetadataHolder with the entity
        let metadata_holder = env.new_object(
            &metadata_holder_class,
            "(Lnet/minestom/server/entity/Entity;)V",
            &[JValue::Object(&display_obj)],
        )?;

        // Create ItemDisplayMeta instance
        let item_display_meta_class = env.find_class("net/minestom/server/entity/metadata/display/ItemDisplayMeta")?;
        let item_display_meta = env.new_object(
            item_display_meta_class,
            "(Lnet/minestom/server/entity/Entity;Lnet/minestom/server/entity/MetadataHolder;)V",
            &[
                JValue::Object(&display_obj),
                JValue::Object(&metadata_holder),
            ],
        )?;

        // Set the item stack
        env.call_method(
            &item_display_meta,
            "setItemStack",
            "(Lnet/minestom/server/item/ItemStack;)V",
            &[JValue::Object(&item.as_obj().as_obj()?)],
        )?;

        // Create the ItemDisplay instance
        let display = Self {
            inner: JavaObject::from_env(&mut env, display_obj)?,
        };

        Ok(display)
    }

    /// Sets the position of this ItemDisplay
    pub fn set_position(&self, x: f64, y: f64, z: f64) -> Result<()> {
        let mut env = get_env()?;
        
        // Create Pos object
        let pos_class = env.find_class("net/minestom/server/coordinate/Pos")?;
        let pos = env.new_object(
            pos_class,
            "(DDD)V",
            &[
                JValue::Double(x),
                JValue::Double(y),
                JValue::Double(z)
            ],
        )?;

        // Call teleport which returns a CompletableFuture
        let future = self.inner.call_object_method(
            "teleport",
            "(Lnet/minestom/server/coordinate/Pos;)Ljava/util/concurrent/CompletableFuture;",
            &[JniValue::Object(pos)],
        )?;

        // Wait for the teleport to complete
        env.call_method(
            future.as_obj()?,
            "join",
            "()Ljava/lang/Object;",
            &[],
        )?;

        Ok(())
    }

    /// Sets the instance this ItemDisplay is in
    pub fn set_instance(&self, instance: &InstanceContainer) -> Result<()> {
        let mut env = get_env()?;
        
        // Call setInstance which returns a CompletableFuture
        let future = self.inner.call_object_method(
            "setInstance",
            "(Lnet/minestom/server/instance/Instance;)Ljava/util/concurrent/CompletableFuture;",
            &[JniValue::Object(instance.inner()?)]
        )?;

        // Call join() on the future to wait for it to complete
        env.call_method(
            future.as_obj()?,
            "join",
            "()Ljava/lang/Object;",
            &[],
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