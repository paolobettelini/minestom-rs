use crate::jni_utils::{get_env, JavaObject, JniValue, ToJava};
use crate::Result;
use jni::objects::{JObject, JValueGen};

#[macro_export]
macro_rules! component {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::text::Component::text(&format!($fmt $(, $arg)*)).expect("Failed to create component")
    };
}

#[derive(Debug, Clone)]
pub struct Component {
    inner: JavaObject,
}

impl Component {
    pub fn text(content: &str) -> Result<Self> {
        let mut env = get_env()?;
        let component_class = env.find_class("net/kyori/adventure/text/Component")?;
        let j_string = content.to_java(&mut env)?;
        
        // Create a text component with explicit style
        let component = env.call_static_method(
            component_class,
            "text",
            "(Ljava/lang/String;)Lnet/kyori/adventure/text/TextComponent;",
            &[j_string.as_jvalue()],
        )?;
        
        Ok(Self {
            inner: JavaObject::from_env(&mut env, component.l()?)?,
        })
    }

    pub fn color(self, color: &str) -> Result<Self> {
        self.create_styled_component(|inner| {
            let mut env = get_env()?;
            let text_color_class = env.find_class("net/kyori/adventure/text/format/TextColor")?;
            let j_string = color.to_java(&mut env)?;
            let text_color = env.call_static_method(
                text_color_class,
                "fromHexString",
                "(Ljava/lang/String;)Lnet/kyori/adventure/text/format/TextColor;",
                &[j_string.as_jvalue()],
            )?;
            let color_obj = text_color.l()?;

            inner.call_object_method(
                "color",
                "(Lnet/kyori/adventure/text/format/TextColor;)Lnet/kyori/adventure/text/Component;",
                &[JniValue::Object(color_obj)],
            )
        })
    }

    // Creates a new component with its own unique style
    fn create_styled_component<F>(&self, style_fn: F) -> Result<Self> 
    where
        F: FnOnce(JavaObject) -> Result<JavaObject>
    {
        // Create a copy of the component with its own style
        let styled_component = style_fn(self.inner.clone())?;
        Ok(Self { inner: styled_component })
    }

    // Color convenience methods
    pub fn red(self) -> Self {
        self.color("#FF0000").expect("Failed to set red color")
    }

    pub fn green(self) -> Self {
        self.color("#00FF00").expect("Failed to set green color")
    }

    pub fn blue(self) -> Self {
        self.color("#0000FF").expect("Failed to set blue color")
    }

    pub fn gold(self) -> Self {
        self.color("#FFAA00").expect("Failed to set gold color")
    }

    pub fn yellow(self) -> Self {
        self.color("#FFFF00").expect("Failed to set yellow color")
    }

    // Style methods
    pub fn bold(self) -> Self {
        self.create_styled_component(|inner| {
            let mut env = get_env().expect("Failed to get JNI environment");
            inner.call_object_method(
                "decoration",
                "(Lnet/kyori/adventure/text/format/TextDecoration;Z)Lnet/kyori/adventure/text/Component;",
                &[
                    JniValue::Object(
                        env.get_static_field(
                            "net/kyori/adventure/text/format/TextDecoration",
                            "BOLD",
                            "Lnet/kyori/adventure/text/format/TextDecoration;",
                        ).expect("Failed to get BOLD decoration")
                        .l().expect("Failed to convert to object")
                    ),
                    JniValue::Bool(true),
                ],
            )
        }).expect("Failed to set bold style")
    }

    pub fn italic(self) -> Self {
        self.create_styled_component(|inner| {
            let mut env = get_env().expect("Failed to get JNI environment");
            inner.call_object_method(
                "decoration",
                "(Lnet/kyori/adventure/text/format/TextDecoration;Z)Lnet/kyori/adventure/text/Component;",
                &[
                    JniValue::Object(
                        env.get_static_field(
                            "net/kyori/adventure/text/format/TextDecoration",
                            "ITALIC",
                            "Lnet/kyori/adventure/text/format/TextDecoration;",
                        ).expect("Failed to get ITALIC decoration")
                        .l().expect("Failed to convert to object")
                    ),
                    JniValue::Bool(true),
                ],
            )
        }).expect("Failed to set italic style")
    }

    // Chain components together
    pub fn chain(self, other: Component) -> Self {
        self.create_styled_component(|inner| {
            let mut env = get_env().expect("Failed to get JNI environment");
            
            // Get the text content class
            let component_class = env.find_class("net/kyori/adventure/text/Component")
                .expect("Failed to find Component class");
                
            // Explicitly join the components with .append() which doesn't inherit styling
            inner.call_object_method(
                "append",
                "(Lnet/kyori/adventure/text/Component;)Lnet/kyori/adventure/text/Component;",
                &[other.inner.to_java(&mut env)
                    .expect("Failed to convert JavaObject to JniValue")],
            )
        }).expect("Failed to chain components")
    }
    
    /// Convenience method to chain with a newline in between
    pub fn chain_newline(self, other: Component) -> Self {
        self.chain(Self::newline()).chain(other)
    }
    
    /// Creates a newline component that can be chained
    pub fn newline() -> Self {
        Self::text("\n").expect("Failed to create newline")
    }

    pub(crate) fn as_jvalue<'local>(
        &self,
        env: &mut jni::JNIEnv<'local>,
    ) -> Result<JniValue<'local>> {
        let obj = self.inner.as_obj()?;
        Ok(JniValue::Object(env.new_local_ref(&obj)?))
    }

    pub fn append(self, text: &str) -> Result<Self> {
        let mut env = get_env()?;
        let result = self.inner.call_object_method(
            "append",
            "(Ljava/lang/String;)Lnet/kyori/adventure/text/TextComponent$Builder;",
            &[text.to_java(&mut env)?],
        )?;
        Ok(Self { inner: result })
    }

    // Reset all styles on this component
    pub fn reset_style(self) -> Result<Self> {
        let mut env = get_env()?;
        let result = self.inner.call_object_method(
            "style",
            "(Lnet/kyori/adventure/text/format/Style;)Lnet/kyori/adventure/text/Component;",
            &[JniValue::Object(
                env.call_static_method(
                    "net/kyori/adventure/text/format/Style",
                    "empty",
                    "()Lnet/kyori/adventure/text/format/Style;",
                    &[],
                )?.l()?
            )],
        )?;
        Ok(Self { inner: result })
    }
}
