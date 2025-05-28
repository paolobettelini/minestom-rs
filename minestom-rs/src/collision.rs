use crate::jni_utils::{JavaObject, get_env};
use crate::{MinestomError, Result};
use jni::objects::JValue;

/// Represents a Minestom BoundingBox, defined by two relative corner vectors.
#[derive(Clone)]
pub struct BoundingBox {
    inner: JavaObject,
}

impl BoundingBox {
    /// Constructs a new BoundingBox with given relative start and end coordinates.
    pub fn new(
        start_x: f64,
        start_y: f64,
        start_z: f64,
        end_x: f64,
        end_y: f64,
        end_z: f64,
    ) -> Result<Self> {
        let mut env = get_env()?;
        // Find the BoundingBox class
        let bb_class = env.find_class("net/minestom/server/collision/BoundingBox")?;
        // Create Vec instances for start and end
        let vec_class = env.find_class("net/minestom/server/coordinate/Vec")?;
        let start_vec = env.new_object(
            &vec_class,
            "(DDD)V",
            &[
                JValue::Double(start_x),
                JValue::Double(start_y),
                JValue::Double(start_z),
            ],
        )?;
        let end_vec = env.new_object(
            &vec_class,
            "(DDD)V",
            &[
                JValue::Double(end_x),
                JValue::Double(end_y),
                JValue::Double(end_z),
            ],
        )?;
        // Construct the BoundingBox
        let bb_obj = env.new_object(
            bb_class,
            "(Lnet/minestom/server/coordinate/Vec;Lnet/minestom/server/coordinate/Vec;)V",
            &[JValue::Object(&start_vec), JValue::Object(&end_vec)],
        )?;
        Ok(BoundingBox {
            inner: JavaObject::from_env(&mut env, bb_obj)?,
        })
    }

    /// Returns the inner JavaObject (for passing to JNI methods).
    pub(crate) fn as_java(&self) -> &JavaObject {
        &self.inner
    }
}
