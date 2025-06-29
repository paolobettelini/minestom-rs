use jni::sys::{jlong, jobject, jstring};
use jni::{
    JNIEnv,
    objects::{JClass, JObject, JValue},
    sys,
};
use minestom::{
    Player, Pos,
    instance::{Instance, InstanceContainer},
};
use minestom::{
    Result,
    jni_utils::{JavaObject, get_env},
};
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    sync::{
        Arc, RwLock,
        atomic::{AtomicU64, Ordering},
    },
};

/// Trait to implement in Rust for any GenericModelImpl subclass
pub trait GenericModel: Send + Sync + 'static {
    fn get_id(&self) -> String;
    fn init(&self, instance: &dyn Instance, pos: Pos);
}

#[derive(Clone)]
pub struct WseeModel {
    inner: JavaObject,
}

// Registry mapping callback IDs to user implementations
static MODEL_REGISTRY: Lazy<RwLock<HashMap<u64, Arc<dyn GenericModel>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));
static NEXT_MODEL_ID: AtomicU64 = AtomicU64::new(1);

// Hardcoded Java subclass for callbacks
const JAVA_CLASS: &str = "rust/wsee/GenericModelCallback";

/// JNI callback for getId()
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_rust_wsee_GenericModelCallback_nativeGetId(
    raw_env: *mut sys::JNIEnv,
    _class: JClass,
    callback_id: jlong,
) -> jstring {
    let env = unsafe { JNIEnv::from_raw(raw_env).unwrap() };
    let id = MODEL_REGISTRY
        .read()
        .unwrap()
        .get(&(callback_id as u64))
        .and_then(|m| Some(m.get_id()))
        .unwrap_or_default();
    env.new_string(id).unwrap().into_raw()
}

/// JNI callback for init(...)
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_rust_wsee_GenericModelCallback_nativeInit(
    raw_env: *mut sys::JNIEnv,
    _class: JClass,
    callback_id: jlong,
    j_instance: jni::objects::JObject,
    j_pos: jni::objects::JObject,
) {
    let mut env = unsafe { JNIEnv::from_raw(raw_env).unwrap() };
    let instance = InstanceContainer::new(JavaObject::from_env(&mut env, j_instance).unwrap());
    let pos = Pos::new(JavaObject::from_env(&mut env, j_pos).unwrap());

    if let Some(model) = MODEL_REGISTRY.read().unwrap().get(&(callback_id as u64)) {
        let _ = model.init(&instance, pos);
    }
}

/// Registers a Rust `GenericModel` impl and returns the Java callback object
pub fn create_wsee_model<M: GenericModel>(model_impl: M) -> Result<WseeModel> {
    // Allocate a new callback ID
    let id = NEXT_MODEL_ID.fetch_add(1, Ordering::SeqCst);
    MODEL_REGISTRY
        .write()
        .unwrap()
        .insert(id, Arc::new(model_impl));

    // Construct the Java GenericModelCallback(long callbackId)
    let mut env = get_env()?;
    let obj = env.new_object(JAVA_CLASS, "(J)V", &[JValue::Long(id as i64)])?;
    Ok(WseeModel {
        inner: JavaObject::from_env(&mut env, obj)?,
    })
}

impl WseeModel {
    pub fn init(&self, instance: &dyn Instance, pos: Pos) -> Result<()> {
        let mut env = get_env()?;
        env.call_method(
            &self.inner.as_obj()?,
            "init",
            "(Lnet/minestom/server/instance/Instance;Lnet/minestom/server/coordinate/Pos;)V",
            &[
                JValue::Object(&instance.inner()?), // your InstanceContainer
                JValue::Object(&pos.inner()?),      // your Pos
            ],
        )?;

        Ok(())
    }

    pub fn inner(&self) -> Result<JObject<'_>> {
        self.inner.as_obj()
    }

    pub fn add_viewer(&self, player: &Player) -> Result<()> {
        let mut env = get_env()?;

        env.call_method(
            &self.inner.as_obj()?,
            "addViewer",
            "(Lnet/minestom/server/entity/Player;)Z",
            &[JValue::Object(&player.inner()?)],
        )?;
        Ok(())
    }

    pub fn remove_viewer(&self, player: &Player) -> Result<()> {
        let mut env = get_env()?;

        env.call_method(
            &self.inner.as_obj()?,
            "removeViewer",
            "(Lnet/minestom/server/entity/Player;)Z",
            &[JValue::Object(&player.inner()?)],
        )?;

        Ok(())
    }
}
