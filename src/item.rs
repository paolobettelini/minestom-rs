use crate::text::Component;
use crate::Result;
use crate::jni_utils::{get_env, JavaObject, JniValue};
use jni::objects::{JObject, JValue};
use crate::material::Material;

#[derive(Debug, Clone)]
pub struct ItemStack {
    inner: JavaObject,
}

impl ItemStack {
    pub fn of(material: Material) -> Result<Self> {
        let mut env = get_env()?;
        let material_obj = env.call_static_method(
            "net/minestom/server/item/Material",
            "fromKey",
            "(Ljava/lang/String;)Lnet/minestom/server/item/Material;",
            &[JValue::from(&env.new_string(material.to_java_name())?)],
        )?.l()?;

        let item = env.call_static_method(
            "net/minestom/server/item/ItemStack",
            "of",
            "(Lnet/minestom/server/item/Material;)Lnet/minestom/server/item/ItemStack;",
            &[JValue::Object(&material_obj)],
        )?;

        Ok(Self { inner: JavaObject::new(env.new_global_ref(item.l()?)?) })
    }

    pub fn with_amount(self, amount: i32) -> Result<Self> {
        let mut env = get_env()?;
        let item = env.call_method(
            self.inner.as_obj()?,
            "withAmount",
            "(I)Lnet/minestom/server/item/ItemStack;",
            &[JValue::Int(amount)],
        )?;

        Ok(Self { inner: JavaObject::new(env.new_global_ref(item.l()?)?) })
    }

    pub fn with_tag<T>(&self, tag: &str, value: T) -> Result<Self> 
    where
        T: for<'a> Into<JValue<'a, 'a>>,
    {
        let mut env = get_env()?;
        
        // Create the Tag object
        let tag_obj = env.call_static_method(
            "net/minestom/server/tag/Tag",
            "String",
            "(Ljava/lang/String;)Lnet/minestom/server/tag/Tag;",
            &[JValue::from(&env.new_string(tag)?)],
        )?.l()?;

        // Call withTag
        let item = env.call_method(
            self.inner.as_obj()?,
            "withTag",
            "(Lnet/minestom/server/tag/Tag;Ljava/lang/Object;)Lnet/minestom/server/item/ItemStack;",
            &[JValue::Object(&tag_obj), value.into()],
        )?;

        Ok(Self { inner: JavaObject::new(env.new_global_ref(item.l()?)?) })
    }

    pub fn with_string_tag(&self, tag: &str, value: &str) -> Result<Self> {
        let mut env = get_env()?;
        
        // Create the Tag object
        let tag_obj = env.call_static_method(
            "net/minestom/server/tag/Tag",
            "String",
            "(Ljava/lang/String;)Lnet/minestom/server/tag/Tag;",
            &[JValue::from(&env.new_string(tag)?)],
        )?.l()?;

        // Create the string value
        let jstring = env.new_string(value)?;

        // Call withTag
        let item = env.call_method(
            self.inner.as_obj()?,
            "withTag",
            "(Lnet/minestom/server/tag/Tag;Ljava/lang/Object;)Lnet/minestom/server/item/ItemStack;",
            &[JValue::Object(&tag_obj), JValue::Object(&jstring)],
        )?;

        Ok(Self { inner: JavaObject::new(env.new_global_ref(item.l()?)?) })
    }

    pub fn with_int_tag(&self, tag: &str, value: i32) -> Result<Self> {
        let mut env = get_env()?;
        
        // Create the Tag object
        let tag_obj = env.call_static_method(
            "net/minestom/server/tag/Tag",
            "Integer",
            "(Ljava/lang/String;)Lnet/minestom/server/tag/Tag;",
            &[JValue::from(&env.new_string(tag)?)],
        )?.l()?;

        // Create the integer value
        let int_value = env.new_object(
            "java/lang/Integer",
            "(I)V",
            &[JValue::Int(value)],
        )?;

        // Call withTag
        let item = env.call_method(
            self.inner.as_obj()?,
            "withTag",
            "(Lnet/minestom/server/tag/Tag;Ljava/lang/Object;)Lnet/minestom/server/item/ItemStack;",
            &[JValue::Object(&tag_obj), JValue::Object(&int_value)],
        )?;

        Ok(Self { inner: JavaObject::new(env.new_global_ref(item.l()?)?) })
    }

    pub fn with_custom_model_data(&self, value: &str) -> Result<Self> {
        let mut env = get_env()?;
        
        // Create empty lists for floats, booleans, and colors
        let empty_list = env.new_object("java/util/ArrayList", "()V", &[])?;
        
        // Create the string list with our value
        let string_list = env.new_object("java/util/ArrayList", "()V", &[])?;
        let jstring = env.new_string(value)?;
        env.call_method(
            &string_list,
            "add",
            "(Ljava/lang/Object;)Z",
            &[JValue::Object(&jstring)],
        )?;

        // Call withCustomModelData
        let item = env.call_method(
            self.inner.as_obj()?,
            "withCustomModelData",
            "(Ljava/util/List;Ljava/util/List;Ljava/util/List;Ljava/util/List;)Lnet/minestom/server/item/ItemStack;",
            &[
                JValue::Object(&empty_list), // floats
                JValue::Object(&empty_list), // booleans
                JValue::Object(&string_list), // strings
                JValue::Object(&empty_list), // colors
            ],
        )?;

        Ok(Self { inner: JavaObject::new(env.new_global_ref(item.l()?)?) })
    }

    pub(crate) fn as_obj(&self) -> &JavaObject {
        &self.inner
    }
}

pub trait InventoryHolder {
    fn get_inventory(&self) -> Result<PlayerInventory>;
}

pub struct PlayerInventory {
    inner: JavaObject,
}

impl PlayerInventory {
    pub(crate) fn from_java(obj: JObject) -> Result<Self> {
        let mut env = get_env()?;
        Ok(Self { inner: JavaObject::new(env.new_global_ref(obj)?) })
    }

    pub fn set_helmet(&self, item: &ItemStack) -> Result<()> {
        let mut env = get_env()?;
        
        // Get the HELMET equipment slot
        let helmet_slot = env.get_static_field(
            "net/minestom/server/entity/EquipmentSlot",
            "HELMET",
            "Lnet/minestom/server/entity/EquipmentSlot;",
        )?.l()?;

        // Get the item object
        let item_obj = item.as_obj().as_obj()?;

        // Call setEquipment with the correct argument types
        env.call_method(
            self.inner.as_obj()?,
            "setEquipment",
            "(Lnet/minestom/server/entity/EquipmentSlot;BLnet/minestom/server/item/ItemStack;)V",
            &[
                JValue::Object(&helmet_slot),
                JValue::Byte(0),
                JValue::Object(&item_obj),
            ],
        )?;

        Ok(())
    }
} 