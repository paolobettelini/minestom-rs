use crate::Result;
use crate::jni_utils::{JavaObject, get_env};
use jni::objects::{JObject, JString, JValue};

/// Handler for entity tags (String-keyed, String-valued tags).
pub struct TagHandler {
    pub(crate) inner: JavaObject,
}

impl TagHandler {
    /// Internal helper: create a net.minestom.server.tag.Tag<String> from a Rust &str.
    fn make_tag(&self, env: &mut jni::JNIEnv, name: &str) -> Result<JavaObject> {
        let jname = env.new_string(name)?;
        let tag_obj = env
            .call_static_method(
                "net/minestom/server/tag/Tag",
                "String",
                "(Ljava/lang/String;)Lnet/minestom/server/tag/Tag;",
                &[JValue::Object(&JObject::from(jname))],
            )?
            .l()?;
        Ok(JavaObject::from_env(env, tag_obj)?)
    }

    /// Reads the specified tag. Returns Some(value) or None if not present.
    pub fn get_tag(&self, key: &str) -> Result<Option<String>> {
        let mut env = get_env()?;
        let tag = self.make_tag(&mut env, key)?;
        let result = env.call_method(
            self.inner.as_obj()?,
            "getTag",
            "(Lnet/minestom/server/tag/Tag;)Ljava/lang/Object;",
            &[JValue::Object(&tag.as_obj()?)],
        )?;
        let obj = result.l()?;
        if obj.is_null() {
            Ok(None)
        } else {
            let jstr = JString::from(obj);
            Ok(Some(env.get_string(&jstr)?.into()))
        }
    }

    /// Returns true if the specified tag is present.
    pub fn has_tag(&self, key: &str) -> Result<bool> {
        let mut env = get_env()?;
        let tag = self.make_tag(&mut env, key)?;
        let flag = env
            .call_method(
                self.inner.as_obj()?,
                "hasTag",
                "(Lnet/minestom/server/tag/Tag;)Z",
                &[JValue::Object(&tag.as_obj()?)],
            )?
            .z()?;
        Ok(flag)
    }

    /// Writes the specified tag value; None to remove.
    pub fn set_tag(&self, key: &str, value: Option<&str>) -> Result<()> {
        let mut env = get_env()?;
        let tag = self.make_tag(&mut env, key)?;
        // Prepare the Java value
        let jobject_val = if let Some(s) = value {
            // New Java String
            let js = env.new_string(s)?;
            JObject::from(js)
        } else {
            JObject::null()
        };
        env.call_method(
            self.inner.as_obj()?,
            "setTag",
            "(Lnet/minestom/server/tag/Tag;Ljava/lang/Object;)V",
            &[JValue::Object(&tag.as_obj()?), JValue::Object(&jobject_val)],
        )?;
        Ok(())
    }

    /// Removes the specified tag.
    pub fn remove_tag(&self, key: &str) -> Result<()> {
        let mut env = get_env()?;
        let tag = self.make_tag(&mut env, key)?;
        env.call_method(
            self.inner.as_obj()?,
            "removeTag",
            "(Lnet/minestom/server/tag/Tag;)V",
            &[JValue::Object(&tag.as_obj()?)],
        )?;
        Ok(())
    }
}
