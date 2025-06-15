use crate::jni_utils::{JavaObject, get_env};
use crate::{instance::Instance, Player, Pos};
use jni::sys::{jboolean, jlong, jobject};
use jni::{
    JNIEnv,
    objects::{JClass, JObject, JValue},
};
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    sync::{
        Arc, RwLock,
        atomic::{AtomicU64, Ordering},
    },
};

use crate::entity::entity::EntityType;

/// Trait that your Rust creature must implement
pub trait EntityCreature: Send + Sync + 'static {
    /// Called when a new player starts seeing this creature
    fn update_new_viewer(&self, player: Player);

    /// Called when a player stops seeing this creature
    fn update_old_viewer(&self, player: Player);

    /// Called every tick; `time` is client‐tick time
    fn tick(&self, time: i64);

    // TODO
    /// Called when damage is applied. Return `true` to let the default damage logic run,
    /// or `false` to cancel.
    // fn damage(&self, damage_type: DamageTypeKey, amount: f32) -> bool;

    /// Called just before the creature is removed
    fn remove(&self);
}

#[derive(Clone)]
pub struct MinestomEntityCreature {
    inner: JavaObject,
}

// Registry mapping callback IDs → the Rust implementation
static CREATURE_REGISTRY: Lazy<RwLock<HashMap<u64, Arc<dyn EntityCreature>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));
static NEXT_CREATURE_ID: AtomicU64 = AtomicU64::new(1);

// Hardcoded Java subclass for callbacks
const JAVA_CLASS: &str = "rust/minestom/EntityCreatureCallback";

/// JNI callback: updateNewViewer(Player)
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_rust_minestom_EntityCreatureCallback_nativeUpdateNewViewer(
    raw_env: *mut jni::sys::JNIEnv,
    _class: JClass,
    callback_id: jlong,
    j_player: jobject,
) {
    let env = unsafe { JNIEnv::from_raw(raw_env).unwrap() };
    let registry = CREATURE_REGISTRY.read().unwrap();
    if let Some(creature) = registry.get(&(callback_id as u64)) {
        let mut env = env;
        let rust_player = Player::new(
            JavaObject::from_env(&mut env, unsafe { JObject::from_raw(j_player) }).unwrap(),
        );
        creature.update_new_viewer(rust_player);
    }
}

/// JNI callback: updateOldViewer(Player)
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_rust_minestom_EntityCreatureCallback_nativeUpdateOldViewer(
    raw_env: *mut jni::sys::JNIEnv,
    _class: JClass,
    callback_id: jlong,
    j_player: jobject,
) {
    let env = unsafe { JNIEnv::from_raw(raw_env).unwrap() };
    let registry = CREATURE_REGISTRY.read().unwrap();
    if let Some(creature) = registry.get(&(callback_id as u64)) {
        let mut env = env;
        let rust_player = Player::new(
            JavaObject::from_env(&mut env, unsafe { JObject::from_raw(j_player) }).unwrap(),
        );
        creature.update_old_viewer(rust_player);
    }
}

/// JNI callback: tick(long time)
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_rust_minestom_EntityCreatureCallback_nativeTick(
    raw_env: *mut jni::sys::JNIEnv,
    _class: JClass,
    callback_id: jlong,
    time: jlong,
) {
    let registry = CREATURE_REGISTRY.read().unwrap();
    if let Some(creature) = registry.get(&(callback_id as u64)) {
        creature.tick(time as i64);
    }
}

/*  TODO
/// JNI callback: damage(DynamicRegistry.Key<DamageType>, float) → boolean
#[unsafe(no_mangle)]
pub unsafe extern "system" fn
Java_rust_minestom_EntityCreatureCallback_nativeDamage(
    raw_env: *mut jni::sys::JNIEnv,
    _class: JClass,
    callback_id: jlong,
    j_damage_key: jobject,
    amount: f32,
) -> jboolean {
    let env = JNIEnv::from_raw(raw_env).unwrap();
    let registry = CREATURE_REGISTRY.read().unwrap();
    if let Some(creature) = registry.get(&(callback_id as u64)) {
        // Wrap the Java DynamicRegistry.Key<DamageType> into our Rust key type
        let mut env = env;
        let rust_key = RegistryKey::new(JavaObject::from_env(&mut env, JObject::from(j_damage_key)).unwrap());
        // Convert to a DamageTypeKey. (You may need to adapt this if your wrapper differs.)
        let damage_key = DamageTypeKey::from_registry_key(rust_key);
        let keep_default = creature.damage(damage_key, amount);
        return if keep_default { 1 } else { 0 };
    }
    // If no entry, default to “let base logic run”
    1
}*/

/// JNI callback: remove()
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_rust_minestom_EntityCreatureCallback_nativeRemove(
    _raw_env: *mut jni::sys::JNIEnv,
    _class: JClass,
    callback_id: jlong,
) {
    let registry = CREATURE_REGISTRY.read().unwrap();
    if let Some(creature) = registry.get(&(callback_id as u64)) {
        creature.remove();
    }
}

/// Registers a Rust `EntityCreature` implementation and returns the Java callback object
///
/// # Arguments
/// * `entity_type` – one of your Rust `EntityType` variants
/// * `creature_impl` – an `Arc<dyn EntityCreature>`
///
/// # Returns
/// A `MinestomEntityCreature` holding the Java `EntityCreatureCallback` instance.
///
pub fn create_entity_creature(
    entity_type: EntityType,
    creature_impl: Arc<dyn EntityCreature>,
) -> crate::Result<MinestomEntityCreature> {
    // 1) Generate a new callback ID and insert the Arc<dyn EntityCreature> into the registry:
    let id = NEXT_CREATURE_ID.fetch_add(1, Ordering::SeqCst);
    CREATURE_REGISTRY.write().unwrap().insert(id, creature_impl);

    // 2) Grab a JNIEnv so we can construct the Java side:
    let mut env = get_env()?;

    // 3) Look up the Java-side EntityType enum (e.g. EntityType.ZOMBIE):
    let et_class = "net/minestom/server/entity/EntityType";
    let field_name = entity_type.to_java_field();
    let sig = "Lnet/minestom/server/entity/EntityType;";
    let java_entity_type = env.get_static_field(et_class, field_name, sig)?.l()?;

    // 4) Construct the Java `new EntityCreatureCallback(long callbackId, EntityType type)`
    let obj = env.new_object(
        JAVA_CLASS,
        "(JLnet/minestom/server/entity/EntityType;)V",
        &[
            JValue::Long(id as i64),
            JValue::Object(&java_entity_type.into()),
        ],
    )?;

    Ok(MinestomEntityCreature {
        inner: JavaObject::from_env(&mut env, obj)?,
    })
}

impl MinestomEntityCreature {
    pub fn null() -> Self {
        MinestomEntityCreature {
            inner: JavaObject::null(),
        }
    }

    pub fn set_invisible(&self, invisible: bool) -> crate::Result<()> {
        let mut env = get_env()?;
        env.call_method(
            &self.inner.as_obj()?,
            "setInvisible",
            "(Z)V",
            &[JValue::Bool(if invisible { 1 } else { 0 })],
        )?;
        Ok(())
    }

    pub fn set_instance_and_pos(
        &self,
        instance: &dyn Instance,
        pos: &Pos,
    ) -> crate::Result<()> {
        let mut env = get_env()?;
        env.call_method(
            &self.inner.as_obj()?,
            "setInstance",
            "(Lnet/minestom/server/instance/Instance;Lnet/minestom/server/coordinate/Pos;)Ljava/util/concurrent/CompletableFuture;",
            &[
                JValue::Object(&instance.inner()?),
                JValue::Object(&pos.inner()?),
            ],
        )?;
        Ok(())
    }

    /// Once you've created the Java callback, you typically want to add it to an instance:
    /// call `.spawn()` or similar.
    ///
    /// This helper simply calls `setInstance(instance, pos)` on your Java object.
    pub fn spawn(&self, instance: &dyn Instance, pos: Pos) -> crate::Result<()> {
        let mut env = get_env()?;
        env.call_method(
            &self.inner.as_obj()?,
            "setInstance",
            "(Lnet/minestom/server/instance/Instance;Lnet/minestom/server/coordinate/Pos;)Ljava/util/concurrent/CompletableFuture;",
            &[
                JValue::Object(&instance.inner()?),
                JValue::Object(&pos.inner()?),
            ],
        )?;
        Ok(())
    }

    // Helper: expose the raw `EntityCreature` Java wrapper,
    // in case you need to call other methods.
    // pub fn as_entity(&self) -> crate::Result<MinestomEntityCreature> {
    //     Ok(MinestomEntityCreature::new(self.inner.clone()))
    // }
}
