use std::fmt;

use crate::coordinate::Position;
use crate::jni_utils::{get_env, JavaObject, JniValue};
use crate::sound::Sound;
use crate::text::Component;
use crate::Result;
use jni::objects::{JObject, JValue};
use crate::attribute::{Attribute, AttributeInstance};
use jni::objects::{JString};

/// Represents a Minecraft game mode
#[derive(Debug, Clone, Copy)]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

impl GameMode {
    fn to_java_name(&self) -> &'static str {
        match self {
            GameMode::Survival => "SURVIVAL",
            GameMode::Creative => "CREATIVE",
            GameMode::Adventure => "ADVENTURE",
            GameMode::Spectator => "SPECTATOR",
        }
    }
}

#[derive(Clone)]
pub struct Player {
    inner: JavaObject,
}

impl fmt::Debug for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Player {{ inner: {:?} }}", self.inner)
    }
}

impl Player {
    pub(crate) fn new(inner: JavaObject) -> Self {
        Self { inner }
    }

    pub fn send_message(&self, message: &Component) -> Result<()> {
        let mut env = get_env()?;
        self.inner.call_void_method(
            "sendMessage",
            "(Lnet/kyori/adventure/text/Component;)V",
            &[message.as_jvalue(&mut env)?],
        )
    }

    /// Gets the username of the player.
    pub fn get_username(&self) -> Result<String> {
        let mut env = get_env()?;
        let result = self
            .inner
            .call_object_method("getUsername", "()Ljava/lang/String;", &[])?;

        let obj = result.as_obj()?;
        let string_ref = jni::objects::JString::from(obj);
        let jstr = env.get_string(&string_ref)?;
        Ok(jstr.to_string_lossy().into_owned())
    }

    /// Sets the player's game mode.
    /// Returns true if the game mode was changed successfully.
    pub fn set_game_mode(&self, game_mode: GameMode) -> Result<bool> {
        let mut env = get_env()?;

        // Find the GameMode enum class
        let game_mode_class = env.find_class("net/minestom/server/entity/GameMode")?;

        // Get the enum constant for the specified game mode
        let game_mode_obj = env.get_static_field(
            game_mode_class,
            game_mode.to_java_name(),
            "Lnet/minestom/server/entity/GameMode;",
        )?;

        // Call setGameMode on the player
        let result = self.inner.call_bool_method(
            "setGameMode",
            "(Lnet/minestom/server/entity/GameMode;)Z",
            &[JniValue::Object(game_mode_obj.l()?)],
        )?;

        Ok(result)
    }

    /// Sets whether this player is allowed to fly
    pub fn set_allow_flying(&self, allow: bool) -> Result<()> {
        self.inner.call_void_method(
            "setAllowFlying",
            "(Z)V",
            &[JniValue::Bool(allow)],
        )
    }

    /// Teleports the player to a specific position with view angles.
    pub fn teleport(&self, x: f64, y: f64, z: f64, yaw: f32, pitch: f32) -> Result<()> {
        let mut env = get_env()?;

        // Create a new Pos object with the coordinates and view angles
        let pos_class = env.find_class("net/minestom/server/coordinate/Pos")?;
        let pos = env.new_object(
            pos_class,
            "(DDDFF)V",
            &[
                JValue::Double(x),
                JValue::Double(y),
                JValue::Double(z),
                JValue::Float(yaw),
                JValue::Float(pitch),
            ],
        )?;

        // Create empty long array for relative elements
        let empty_array = env.new_long_array(0)?;

        // Call teleport on the player
        let future = self.inner.call_object_method(
            "teleport",
            "(Lnet/minestom/server/coordinate/Pos;[JI)Ljava/util/concurrent/CompletableFuture;",
            &[
                JniValue::Object(pos),
                JniValue::Object(JObject::from(empty_array)),
                JniValue::Int(0), // No special flags
            ],
        )?;

        // Wait for the teleport to complete
        let future_obj = future.as_obj()?;
        env.call_method(future_obj, "join", "()Ljava/lang/Object;", &[])?;

        Ok(())
    }

    pub fn play_sound(&self, sound: &Sound) -> Result<()> {
        let mut env = get_env()?;
        self.inner.call_void_method(
            "playSound",
            "(Lnet/kyori/adventure/sound/Sound;)V",
            &[sound.as_jvalue(&mut env)?],
        )
    }

    /// Gets the current position of the player.
    pub fn get_position(&self) -> Result<Position> {
        let mut env = get_env()?;
        let result = self.inner.call_object_method(
            "getPosition",
            "()Lnet/minestom/server/coordinate/Pos;",
            &[],
        )?;

        let obj = result.as_obj()?;
        let x = env.call_method(&obj, "x", "()D", &[])?.d()?;
        let y = env.call_method(&obj, "y", "()D", &[])?.d()?;
        let z = env.call_method(&obj, "z", "()D", &[])?.d()?;

        Ok(Position::new(x, y, z))
    }

    /// Gets an attribute instance for the specified attribute
    pub fn get_attribute(&self, attribute: Attribute) -> Result<AttributeInstance> {
        let mut env = get_env()?;
        let j_attribute = attribute.to_java_attribute()?;
        
        let result = self.inner.call_object_method(
            "getAttribute",
            "(Lnet/minestom/server/entity/attribute/Attribute;)Lnet/minestom/server/entity/attribute/AttributeInstance;",
            &[j_attribute.as_jvalue(&mut env)?],
        )?;
        
        Ok(AttributeInstance::new(JavaObject::from_env(&mut env, result.as_obj()?)?))
    }
}
