use jni::objects::{JObject, JString, JValue};
use minestom::jni_utils::{get_env, JavaObject, JniValue};

use crate::generic_model::WseeModel;

#[derive(Debug, Clone)]
pub struct AnimationHandler {
    inner: JavaObject,
}

impl AnimationHandler {
    pub fn new(model: &WseeModel) -> minestom::Result<Self> {
        let mut env = get_env()?;
        let model_obj: JObject = model.inner()?;
        let handler_obj = env.new_object(
            "net/worldseed/multipart/animations/AnimationHandlerImpl",
            "(Lnet/worldseed/multipart/GenericModel;)V",
            &[JValue::Object(&model_obj)],
        )?;
        let handler = JavaObject::from_env(&mut env, handler_obj)?;
        Ok(Self { inner: handler })
    }

    /// Calls the Java method `playRepeat(String animation)` on AnimationHandlerImpl.
    pub fn play_repeat(&self, animation: &str) -> minestom::Result<()> {
        let mut env = get_env()?;
        // Create a Java String from the Rust &str
        let jstr: JString = env.new_string(animation)?;
        // Call `this.inner.playRepeat(jstr)`
        self.inner.call_void_method(
            "playRepeat",
            "(Ljava/lang/String;)V",
            &[JniValue::Object(jni::objects::JObject::from(jstr))],
        )?;
        Ok(())
    }

    /// Calls the Java method `stopRepeat(String animation)` on AnimationHandlerImpl.
    pub fn stop_repeat(&self, animation: &str) -> minestom::Result<()> {
        let mut env = get_env()?;
        let jstr: JString = env.new_string(animation)?;
        self.inner.call_void_method(
            "stopRepeat",
            "(Ljava/lang/String;)V",
            &[JniValue::Object(jni::objects::JObject::from(jstr))],
        )?;
        Ok(())
    }

}