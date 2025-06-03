use crate::Result;
use crate::jni_utils::{JavaObject, JniValue, ToJava, get_env};
use crate::text::Component;
use crate::entity::Player;
use crate::material::Material;
use jni::objects::{JObject, JValue, JValueGen};

/// Frame types for advancements (TASK, GOAL, CHALLENGE)
#[derive(Debug, Clone)]
pub struct FrameType {
    inner: JavaObject,
}

impl FrameType {
    fn java_class() -> &'static str {
        "net/minestom/server/advancements/FrameType"
    }

    fn from_static(field: &str) -> Result<Self> {
        let mut env = get_env()?;
        let class = env.find_class(Self::java_class())?;
        let val = env.get_static_field(class, field, &format!("L{};", Self::java_class()))?;
        let obj = val.l()?;
        let global = env.new_global_ref(obj)?;
        Ok(FrameType { inner: JavaObject::new(global) })
    }

    pub fn TASK() -> Self { Self::from_static("TASK").unwrap() }
    pub fn GOAL() -> Self { Self::from_static("GOAL").unwrap() }
    pub fn CHALLENGE() -> Self { Self::from_static("CHALLENGE").unwrap() }
}

/// Root node of an advancement tab
#[derive(Debug, Clone)]
pub struct AdvancementRoot {
    inner: JavaObject,
}
impl AdvancementRoot {
    /// Create a new AdvancementRoot
    pub fn new(
        title: &Component,
        description: &Component,
        icon: Material,
        frame: FrameType,
        x: f32,
        y: f32,
        background: &str,
    ) -> Result<Self> {
        let mut env = get_env()?;
        let class = env.find_class("net/minestom/server/advancements/AdvancementRoot")?;

        // Prepare common values
        let title_val = title.as_jvalue(&mut env)?;
        let desc_val = description.as_jvalue(&mut env)?;
        // Material -> Java Material
        let mat_str = env.new_string(icon.to_java_name())?;
        let material_obj = env.call_static_method(
            "net/minestom/server/item/Material",
            "fromKey",
            "(Ljava/lang/String;)Lnet/minestom/server/item/Material;",
            &[(&mat_str).into()],
        )?.l()?;
        let binding = JniValue::Object(material_obj);
        let material_val = binding.as_jvalue();
        // FrameType
        let frame_obj = frame.inner.as_obj()?;
        let binding = JniValue::Object(frame_obj);
        let frame_val = binding.as_jvalue();
        let binding = JniValue::Float(x);
        let x_val = binding.as_jvalue();
        let binding = JniValue::Float(y);
        let y_val = binding.as_jvalue();

        // Construct
        let bg_str = env.new_string(background)?;
        let obj = env.new_object(
            class,
            "(Lnet/kyori/adventure/text/Component;Lnet/kyori/adventure/text/Component;Lnet/minestom/server/item/Material;Lnet/minestom/server/advancements/FrameType;FFLjava/lang/String;)V",
            &[ title_val.as_jvalue(), desc_val.as_jvalue(), material_val, frame_val, x_val, y_val, (&bg_str).into() ],
        )?;

        let global = env.new_global_ref(obj)?;
        Ok(AdvancementRoot { inner: JavaObject::new(global) })
    }

    pub fn as_advancement(&self) -> Advancement {
        Advancement { inner: self.inner.clone() }
    }

    pub fn inner(&self) -> &JavaObject { &self.inner }
}

/// Manager for advancement tabs
#[derive(Debug, Clone)]
pub struct AdvancementManager {
    pub(crate) inner: JavaObject,
}
impl AdvancementManager {
    /// Create a new tab under this manager
    pub fn create_tab(&self, id: &str, root: AdvancementRoot) -> Result<AdvancementTab> {
        let mut env = get_env()?;
        let id_str = env.new_string(id)?;
        let root_obj = root.inner.as_obj()?;
        let result = env.call_method(
            &self.inner.as_obj()?,
            "createTab",
            "(Ljava/lang/String;Lnet/minestom/server/advancements/AdvancementRoot;)Lnet/minestom/server/advancements/AdvancementTab;",
            &[ (&id_str).into(), JValue::Object(&root_obj) ],
        )?;
        let tab_obj = result.l()?;
        let global = env.new_global_ref(tab_obj)?;
        Ok(AdvancementTab { inner: JavaObject::new(global) })
    }
}

/// A single advancement tab in the client GUI
#[derive(Debug, Clone)]
pub struct AdvancementTab {
    inner: JavaObject,
}
impl AdvancementTab {
    /// Create a sub-advancement under this tab
    pub fn create_advancement(&self, id: &str, adv: Advancement, parent: Advancement) -> Result<()> {
        let mut env = get_env()?;
        let id_str = env.new_string(id)?;
        let adv_obj = adv.inner.as_obj()?;
        let parent_obj = parent.inner.as_obj()?;
        env.call_method(
            &self.inner.as_obj()?,
            "createAdvancement",
            "(Ljava/lang/String;Lnet/minestom/server/advancements/Advancement;Lnet/minestom/server/advancements/Advancement;)V",
            &[ JValue::Object(&id_str.into()), JValue::Object(&adv_obj), JValue::Object(&parent_obj) ],
        )?;
        Ok(())
    }

    /// Show this tab to a player
    pub fn add_viewer(&self, player: &Player) -> Result<bool> {
        let mut env = get_env()?;
        let player_obj = player.inner.as_obj()?;
        // call Java boolean addViewer(Player)
        let added = env.call_method(
            &self.inner.as_obj()?,
            "addViewer",
            "(Lnet/minestom/server/entity/Player;)Z",
            &[JValue::Object(&player_obj)],
        )?.z()?;
        Ok(added)
    }
}

/// A child advancement entry
#[derive(Debug, Clone)]
pub struct Advancement {
    inner: JavaObject,
}
impl Advancement {
    /// Build a new advancement node (not yet added to tab)
    pub fn new(
        title: &Component,
        description: &Component,
        icon: Material,
        frame: FrameType,
        x: f32,
        y: f32
    ) -> Result<Self> {
        let mut env = get_env()?;
        let class = env.find_class("net/minestom/server/advancements/Advancement")?;
        let sig = "(Lnet/kyori/adventure/text/Component;Lnet/kyori/adventure/text/Component;Lnet/minestom/server/item/Material;Lnet/minestom/server/advancements/FrameType;FF)V";

        let title_val = title.as_jvalue(&mut env)?;
        let desc_val = description.as_jvalue(&mut env)?;
        // Material -> Java Material
        let mat_str = env.new_string(icon.to_java_name())?;
        let material_obj = env.call_static_method(
            "net/minestom/server/item/Material",
            "fromKey",
            "(Ljava/lang/String;)Lnet/minestom/server/item/Material;",
            &[(&mat_str).into()],
        )?.l()?;
        let material_val = JValueGen::Object(material_obj);
        // Frame
        let frame_obj = frame.inner.as_obj()?;
        let frame_val = JValueGen::Object(frame_obj);
        let x_val = JValueGen::Float(x);
        let y_val = JValueGen::Float(y);

        let obj = env.new_object(
            class,
            sig,
            &[ title_val.as_jvalue(), desc_val.as_jvalue(), (&material_val).into(), (&frame_val).into(), x_val, y_val ],
        )?;
        let global = env.new_global_ref(obj)?;
        Ok(Advancement { inner: JavaObject::new(global) })
    }

    /// Show or hide the toast notification
    pub fn show_toast(&self, show: bool) -> Result<Advancement> {
        let mut env = get_env()?;
        let result = env.call_method(
            &self.inner.as_obj()?,
            "showToast",
            "(Z)Lnet/minestom/server/advancements/Advancement;",
            &[ JValue::Bool(if show { 1 } else { 0 }) ],
        )?;
        let adv_obj = result.l()?;
        let global = env.new_global_ref(adv_obj)?;
        Ok(Advancement { inner: JavaObject::new(global) })
    }

    /// Mark this advancement as achieved or unachieved
pub fn set_achieved(&self, achieved: bool) -> Result<Advancement> {
    let mut env = get_env()?;
    // Chiama Java: Advancement setAchieved(boolean)
    let result = env.call_method(
        &self.inner.as_obj()?,
        "setAchieved",
        "(Z)Lnet/minestom/server/advancements/Advancement;",
        &[ JValue::Bool(if achieved { 1 } else { 0 }) ],
    )?;
    let adv_obj = result.l()?;
    let global = env.new_global_ref(adv_obj)?;
    Ok(Advancement { inner: JavaObject::new(global) })
}

    pub fn inner(&self) -> &JavaObject { &self.inner }
}
