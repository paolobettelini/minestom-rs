use crate::text::Component;
use crate::Result;
use crate::jni_utils::{get_env, JavaObject, JniValue};
use uuid::Uuid;
use jni::objects::{JObject, JValue};
use std::str::FromStr;

pub struct ResourcePackInfo {
    info: JavaObject,
}

impl ResourcePackInfo {
    pub fn new(uuid: Uuid, url: &str, hash: &str) -> Result<Self> {
        let mut env = get_env()?;
        let url_uri = env.call_static_method(
            "java/net/URI",
            "create",
            "(Ljava/lang/String;)Ljava/net/URI;",
            &[JValue::from(&env.new_string(url)?)],
        )?.l()?;

        // Create a builder
        let builder = env.call_static_method(
            "net/kyori/adventure/resource/ResourcePackInfo",
            "resourcePackInfo",
            "()Lnet/kyori/adventure/resource/ResourcePackInfo$Builder;",
            &[],
        )?;

        let builder_obj = builder.l()?;

        // Set the UUID
        let uuid_obj = env.call_static_method(
            "java/util/UUID",
            "fromString",
            "(Ljava/lang/String;)Ljava/util/UUID;",
            &[JValue::from(&env.new_string(uuid.to_string())?)],
        )?.l()?;

        let builder_obj = env.call_method(
            builder_obj,
            "id",
            "(Ljava/util/UUID;)Lnet/kyori/adventure/resource/ResourcePackInfo$Builder;",
            &[JValue::Object(&uuid_obj)],
        )?.l()?;

        // Set the URL
        let builder_obj = env.call_method(
            builder_obj,
            "uri",
            "(Ljava/net/URI;)Lnet/kyori/adventure/resource/ResourcePackInfo$Builder;",
            &[JValue::Object(&url_uri)],
        )?.l()?;

        // Set the hash
        let builder_obj = env.call_method(
            builder_obj,
            "hash",
            "(Ljava/lang/String;)Lnet/kyori/adventure/resource/ResourcePackInfo$Builder;",
            &[JValue::from(&env.new_string(hash)?)],
        )?.l()?;

        // Build the ResourcePackInfo
        let info = env.call_method(
            builder_obj,
            "build",
            "()Lnet/kyori/adventure/resource/ResourcePackInfo;",
            &[],
        )?;

        Ok(Self { info: JavaObject::new(env.new_global_ref(info.l()?)?) })
    }

    pub fn as_obj(&self) -> &JavaObject {
        &self.info
    }
}

pub struct ResourcePackRequest {
    request: JavaObject,
}

pub struct ResourcePackRequestBuilder {
    builder: JavaObject,
}

impl ResourcePackRequestBuilder {
    pub fn new() -> Result<Self> {
        let mut env = get_env()?;
        let builder = env.call_static_method(
            "net/kyori/adventure/resource/ResourcePackRequest",
            "resourcePackRequest",
            "()Lnet/kyori/adventure/resource/ResourcePackRequest$Builder;",
            &[],
        )?;

        Ok(Self { builder: JavaObject::new(env.new_global_ref(builder.l()?)?) })
    }

    pub fn packs(self, pack: ResourcePackInfo) -> Result<Self> {
        let mut env = get_env()?;
        let pack_obj = pack.as_obj().as_obj()?;
        let empty_array = env.new_object_array(
            0,
            "net/kyori/adventure/resource/ResourcePackInfoLike",
            JObject::null(),
        )?;

        let builder = env.call_method(
            self.builder.as_obj()?,
            "packs",
            "(Lnet/kyori/adventure/resource/ResourcePackInfoLike;[Lnet/kyori/adventure/resource/ResourcePackInfoLike;)Lnet/kyori/adventure/resource/ResourcePackRequest$Builder;",
            &[JValue::Object(&pack_obj), JValue::Object(&empty_array)],
        )?;

        Ok(Self { builder: JavaObject::new(env.new_global_ref(builder.l()?)?) })
    }

    pub fn prompt(self, message: &Component) -> Result<Self> {
        let mut env = get_env()?;
        let jvalue = message.as_jvalue(&mut env)?;
        let builder = env.call_method(
            self.builder.as_obj()?,
            "prompt",
            "(Lnet/kyori/adventure/text/Component;)Lnet/kyori/adventure/resource/ResourcePackRequest$Builder;",
            &[jvalue.as_jvalue()],
        )?;

        Ok(Self { builder: JavaObject::new(env.new_global_ref(builder.l()?)?) })
    }

    pub fn required(self, required: bool) -> Result<Self> {
        let mut env = get_env()?;
        let builder = env.call_method(
            self.builder.as_obj()?,
            "required",
            "(Z)Lnet/kyori/adventure/resource/ResourcePackRequest$Builder;",
            &[JValue::Bool(if required { 1 } else { 0 })],
        )?;

        Ok(Self { builder: JavaObject::new(env.new_global_ref(builder.l()?)?) })
    }

    pub fn build(self) -> Result<ResourcePackRequest> {
        let mut env = get_env()?;
        let request = env.call_method(
            self.builder.as_obj()?,
            "build",
            "()Lnet/kyori/adventure/resource/ResourcePackRequest;",
            &[],
        )?;

        Ok(ResourcePackRequest { request: JavaObject::new(env.new_global_ref(request.l()?)?) })
    }
}

impl ResourcePackRequest {
    pub fn as_obj(&self) -> &JavaObject {
        &self.request
    }
} 