use std::fmt;
use std::sync::Arc;

pub mod display;
pub mod entity;
pub mod entity_creature;
pub mod player;

pub use display::*;
pub use entity_creature::*;
pub use player::*;

use crate::Result;
use crate::attribute::{Attribute, AttributeInstance};
use crate::coordinate::Position;
use crate::jni_utils::{JavaObject, JniValue, get_env};
use crate::sound::Sound;
use crate::text::Component;
use jni::objects::JString;
use jni::objects::{JObject, JValue};
use uuid;

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
    pub(crate) inner: JavaObject,
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

    /// Gets the UUID of the player.
    pub fn get_uuid(&self) -> Result<uuid::Uuid> {
        let mut env = get_env()?;

        // First get the identity
        let identity = self.inner.call_object_method(
            "identity",
            "()Lnet/kyori/adventure/identity/Identity;",
            &[],
        )?;

        // Then get the UUID from the identity
        let uuid_result = env.call_method(identity.as_obj()?, "uuid", "()Ljava/util/UUID;", &[])?;

        let uuid_obj = uuid_result.l()?;

        // Convert Java UUID to String
        let uuid_str = env.call_method(uuid_obj, "toString", "()Ljava/lang/String;", &[])?;

        let uuid_jstring = JString::from(uuid_str.l()?);
        let uuid_rust_str = env.get_string(&uuid_jstring)?;

        // Parse the UUID string into a Rust UUID
        Ok(uuid::Uuid::parse_str(&uuid_rust_str.to_string_lossy())?)
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
        self.inner
            .call_void_method("setAllowFlying", "(Z)V", &[JniValue::Bool(allow)])
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

    pub fn play_sound_at(&self, sound: &Sound, x: f64, y: f64, z: f64) -> Result<()> {
        let mut env = get_env()?;
        self.inner.call_void_method(
            "playSound",
            "(Lnet/kyori/adventure/sound/Sound;DDD)V",
            &[
                sound.as_jvalue(&mut env)?,
                JniValue::Double(x),
                JniValue::Double(y),
                JniValue::Double(z),
            ],
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

        Ok(AttributeInstance::new(JavaObject::from_env(
            &mut env,
            result.as_obj()?,
        )?))
    }
}

/// Represents a player's skin data
#[derive(Debug, Clone)]
pub struct PlayerSkin {
    inner: JavaObject,
}

impl PlayerSkin {
    pub(crate) fn new(inner: JavaObject) -> Self {
        Self { inner }
    }

    /// Creates a new PlayerSkin instance with the given texture value and signature
    pub fn create(texture_value: &str, signature: &str) -> Result<Self> {
        let mut env = get_env()?;
        let skin_class = env.find_class("net/minestom/server/entity/PlayerSkin")?;

        let texture_str = env.new_string(texture_value)?;
        let signature_str = env.new_string(signature)?;

        let skin_obj = env.new_object(
            skin_class,
            "(Ljava/lang/String;Ljava/lang/String;)V",
            &[JValue::Object(&texture_str), JValue::Object(&signature_str)],
        )?;

        Ok(Self::new(JavaObject::from_env(&mut env, skin_obj)?))
    }

    pub(crate) fn inner(&self) -> &JavaObject {
        &self.inner
    }
}
