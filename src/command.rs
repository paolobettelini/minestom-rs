use crate::jni_utils::{get_env, JavaObject, JniValue, ToJava};
use crate::Result;
use jni::objects::JValueGen;

pub trait Command {
    fn name(&self) -> &str;
    fn execute(&self) -> Result<()>;
}

pub struct CommandManager {
    inner: JavaObject,
}

impl CommandManager {
    pub(crate) fn new(inner: JavaObject) -> Self {
        Self { inner }
    }

    pub fn register<T: Command>(&self, command: &T) -> Result<()> {
        let mut env = get_env()?;
        let command_class = env.find_class("net/minestom/server/command/builder/Command")?;
        let j_string = command.name().to_java(&mut env)?;
        let command_obj = env.new_object(
            command_class,
            "(Ljava/lang/String;)V",
            &[j_string.as_jvalue()],
        )?;

        self.inner.call_void_method(
            "register",
            "(Lnet/minestom/server/command/builder/Command;)V",
            &[JniValue::from(command_obj)],
        )
    }
}
