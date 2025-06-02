use crate::Component;
use crate::InstanceContainer;
use crate::Result;
use crate::collision::BoundingBox;
use crate::jni_utils::{JavaObject, JniValue, get_env};
use crate::tag::TagHandler;
use jni::objects::{JObject, JValue};
use uuid::Uuid;

/// Represents the available Minestom entity types for creation.
#[derive(Debug, Clone, Copy)]
pub enum EntityType {
    ArmorStand,
    Player,
    Zombie,
    // ...
}

impl EntityType {
    /// Returns the corresponding Java static field name for this variant.
    pub fn to_java_field(&self) -> &'static str {
        match self {
            EntityType::ArmorStand => "ARMOR_STAND",
            EntityType::Player => "PLAYER",
            EntityType::Zombie => "ZOMBIE",
            // ...
        }
    }

    pub fn from_java_name(name: &str) -> Option<Self> {
        match name {
            "minecraft:armor_stand" => Some(EntityType::ArmorStand),
            "minecraft:player" => Some(EntityType::Player),
            "minecraft:zombie" => Some(EntityType::Zombie),
            _ => panic!("Unknown EntityType: {}", name),
        }
    }
}

/// A generic wrapper around a Minestom entity Java object.
#[derive(Clone)]
pub struct Entity {
    inner: JavaObject,
}

impl Entity {
    /// Constructs a new `Entity` from a `JavaObject`.
    pub fn new(inner: JavaObject) -> Self {
        Self { inner }
    }

    /// Creates a new entity of the given `EntityType`.
    pub fn new_from_type(entity_type: EntityType) -> Result<Self> {
        let mut env = get_env()?;

        // Get the EntityType class
        let et_class = env.find_class("net/minestom/server/entity/EntityType")?;
        let field_name = entity_type.to_java_field();
        // Retrieve the static field matching our variant
        let et_obj = env
            .get_static_field(
                et_class,
                field_name,
                "Lnet/minestom/server/entity/EntityType;",
            )?
            .l()?;

        // Instantiate the Java Entity with the given type
        let entity_class = env.find_class("net/minestom/server/entity/Entity")?;
        let entity_obj = env.new_object(
            entity_class,
            "(Lnet/minestom/server/entity/EntityType;)V",
            &[JValue::Object(&JObject::from(et_obj))],
        )?;

        Ok(Self {
            inner: JavaObject::from_env(&mut env, entity_obj)?,
        })
    }

    /// Sets whether this Entity should be affected by gravity
    pub fn set_no_gravity(&self, no_gravity: bool) -> Result<()> {
        let mut env = get_env()?;

        // Get the entity metadata
        let meta_obj = self.inner.call_object_method(
            "getEntityMeta",
            "()Lnet/minestom/server/entity/metadata/EntityMeta;",
            &[],
        )?;

        // Call setHasNoGravity(boolean)
        env.call_method(
            &meta_obj.as_obj()?,
            "setHasNoGravity",
            "(Z)V",
            &[JValue::Bool(if no_gravity { 1 } else { 0 })],
        )?;

        Ok(())
    }

    /// Sets whether this Entity should be visible
    pub fn set_invisible(&self, invisible: bool) -> Result<()> {
        let mut env = get_env()?;

        // Get the entity metadata
        let meta_obj = self.inner.call_object_method(
            "getEntityMeta",
            "()Lnet/minestom/server/entity/metadata/EntityMeta;",
            &[],
        )?;

        // Call setInvisible(boolean)
        env.call_method(
            &meta_obj.as_obj()?,
            "setInvisible",
            "(Z)V",
            &[JValue::Bool(if invisible { 1 } else { 0 })],
        )?;

        Ok(())
    }

    /// Spawns the entity at the specified location
    pub fn spawn(
        &self,
        instance: &InstanceContainer,
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
    ) -> Result<()> {
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
        env.call_method(future.as_obj()?, "join", "()Ljava/lang/Object;", &[])?;

        Ok(())
    }

    pub fn set_bounding_box(&self, box_: &BoundingBox) -> Result<()> {
        let mut env = get_env()?;
        let entity_obj = self.inner.as_obj()?;
        env.call_method(
            entity_obj,
            "setBoundingBox",
            "(Lnet/minestom/server/collision/BoundingBox;)V",
            &[JValue::Object(&box_.as_java().as_obj()?)],
        )?;
        Ok(())
    }

    /// Retrieves the UUID of this entity.
    pub fn get_uuid(&self) -> crate::Result<Uuid> {
        // Prepare JNI environment
        let mut env = get_env()?;
        // Get the underlying Java Entity object
        let entity_obj = self.inner.as_obj()?;
        // Call getUuid(): java.util.UUID
        let uuid_j = env.call_method(entity_obj, "getUuid", "()Ljava/util/UUID;", &[])?;
        let uuid_obj = uuid_j.l()?;

        // Extract the two long fields: most and least significant bits
        let msb = env
            .call_method(&uuid_obj, "getMostSignificantBits", "()J", &[])?
            .j()?;
        let lsb = env
            .call_method(&uuid_obj, "getLeastSignificantBits", "()J", &[])?
            .j()?;

        // Combine into a u128: msb << 64 | (lsb as u64)
        let raw = ((msb as u128) << 64) | ((lsb as u64) as u128);
        Ok(Uuid::from_u128(raw))
    }

    /// Returns the `EntityType` of this entity instance.
    pub fn get_type(&self) -> Result<EntityType> {
        let mut env = get_env()?;
        // Call Java's getEntityType()
        let et_value = env.call_method(
            self.inner.as_obj()?,
            "getEntityType",
            "()Lnet/minestom/server/entity/EntityType;",
            &[],
        )?;
        let et_obj = et_value.l()?;
        // Call name() on the EntityType enum
        let name_j = env.call_method(&et_obj, "name", "()Ljava/lang/String;", &[])?;
        let jstr = name_j.l()?;
        let rust_str: String = env.get_string((&jstr).into())?.into();
        Ok(EntityType::from_java_name(&rust_str).unwrap())
    }

    /// Gets the custom name of this entity, if set.
    pub fn get_custom_name(&self) -> Result<Option<Component>> {
        let mut env = get_env()?;
        let name_val = env.call_method(
            self.inner.as_obj()?,
            "getCustomName",
            "()Lnet/kyori/adventure/text/Component;",
            &[],
        )?;
        let obj = name_val.l()?;
        if obj.is_null() {
            Ok(None)
        } else {
            Ok(Some(Component::from_java_object(JavaObject::from_env(
                &mut env, obj,
            )?)))
        }
    }

    /// Sets the custom name of this entity.
    pub fn set_custom_name(&self, name: &Component) -> Result<()> {
        // Use Entity's helper to accept JniValue directly
        let mut env = get_env()?;
        let jval = name.as_jvalue(&mut env)?;
        self.inner.call_void_method(
            "setCustomName",
            "(Lnet/kyori/adventure/text/Component;)V",
            &[jval],
        )?;
        Ok(())
    }

    /// Sets whether the custom name is visible.
    pub fn set_custom_name_visible(&self, visible: bool) -> Result<()> {
        let mut env = get_env()?;
        env.call_method(
            self.inner.as_obj()?,
            "setCustomNameVisible",
            "(Z)V",
            &[JValue::Bool(visible as u8)],
        )?;
        Ok(())
    }

    pub fn tag_handler(&self) -> Result<TagHandler> {
        let mut env = get_env()?;
        // Chiama Java: entity.tagHandler()
        let th_obj = env
            .call_method(
                self.inner.as_obj()?,
                "tagHandler",
                "()Lnet/minestom/server/tag/TagHandler;",
                &[],
            )?
            .l()?;
        Ok(TagHandler {
            inner: JavaObject::from_env(&mut env, th_obj)?,
        })
    }
}
