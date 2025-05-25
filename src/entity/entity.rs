use crate::jni_utils::{get_env, JavaObject};
use uuid::Uuid;

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

    /// Retrieves the UUID of this entity.
    pub fn get_uuid(&self) -> crate::Result<Uuid> {
        // Prepare JNI environment
        let mut env = get_env()?;
        // Get the underlying Java Entity object
        let entity_obj = self.inner.as_obj()?;
        // Call getUuid(): java.util.UUID
        let uuid_j = env.call_method(
            entity_obj,
            "getUuid",
            "()Ljava/util/UUID;",
            &[],
        )?;
        let uuid_obj = uuid_j.l()?;

        // Extract the two long fields: most and least significant bits
        let msb = env.call_method(
            &uuid_obj,
            "getMostSignificantBits",
            "()J",
            &[],
        )?.j()?;
        let lsb = env.call_method(
            &uuid_obj,
            "getLeastSignificantBits",
            "()J",
            &[],
        )?.j()?;

        // Combine into a u128: msb << 64 | (lsb as u64)
        let raw = ((msb as u128) << 64) | ((lsb as u64) as u128);
        Ok(Uuid::from_u128(raw))
    }
}
