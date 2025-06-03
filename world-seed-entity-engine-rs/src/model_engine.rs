use jni::objects::{JObject, JValue};
use minestom::{
    Result,
    jni_utils::{JavaObject, get_env},
    material::Material,
};
use std::path::Path;

pub struct ModelEngine {
    inner: JavaObject,
}

impl ModelEngine {
    pub fn set_model_material(model_material: Material) -> Result<()> {
        let mut env = get_env()?;
        let material_obj = env
            .call_static_method(
                "net/minestom/server/item/Material",
                "fromKey",
                "(Ljava/lang/String;)Lnet/minestom/server/item/Material;",
                &[JValue::from(
                    &env.new_string(model_material.to_java_name())?,
                )],
            )?
            .l()?;

        // Call ModelEngine.setModelMaterial(Material)
        env.call_static_method(
            "net/worldseed/multipart/ModelEngine",
            "setModelMaterial",
            "(Lnet/minestom/server/item/Material;)V",
            &[JValue::Object(&material_obj)],
        )?;

        Ok(())
    }

    pub fn load_mappings<A, B>(mappings_path: A, models_path: B) -> Result<()>
    where
        A: AsRef<Path>,
        B: AsRef<Path>,
    {
        let mut env = get_env()?;

        // Create a FileInputStream for the mappings file
        let mapping_str = mappings_path
            .as_ref()
            .to_str()
            .expect("Invalid mappings path");
        let j_mapping_path = env.new_string(mapping_str)?;
        let fis_obj = env.new_object(
            "java/io/FileInputStream",
            "(Ljava/lang/String;)V",
            &[JValue::Object(&j_mapping_path.into())],
        )?;

        // Wrap it into an InputStreamReader
        let reader_obj = env.new_object(
            "java/io/InputStreamReader",
            "(Ljava/io/InputStream;)V",
            &[JValue::Object(&fis_obj)],
        )?;

        // Convert the models path to a java.nio.file.Path via Paths.get(String...)
        let models_str = models_path.as_ref().to_str().expect("Invalid models path");
        let j_models_str = env.new_string(models_str)?;
        // Create an empty String[] for varargs (no init element)
        let empty_str_array = env.new_object_array(0, "java/lang/String", JObject::null())?;
        let path_obj = env
            .call_static_method(
                "java/nio/file/Paths",
                "get",
                "(Ljava/lang/String;[Ljava/lang/String;)Ljava/nio/file/Path;",
                &[
                    JValue::Object(&j_models_str),
                    JValue::Object(&empty_str_array),
                ],
            )?
            .l()?;

        // Call ModelEngine.loadMappings(Reader, Path)
        env.call_static_method(
            "net/worldseed/multipart/ModelEngine",
            "loadMappings",
            "(Ljava/io/Reader;Ljava/nio/file/Path;)V",
            &[JValue::Object(&reader_obj), JValue::Object(&path_obj)],
        )?;

        Ok(())
    }
}
