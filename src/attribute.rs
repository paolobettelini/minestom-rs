use crate::jni_utils::{get_env, JavaObject, JniValue};
use crate::Result;

#[derive(Debug, Clone, Copy)]
pub enum Attribute {
    Armor,
    ArmorToughness,
    AttackDamage,
    AttackKnockback,
    AttackSpeed,
    BlockBreakSpeed,
    BlockInteractionRange,
    BurningTime,
    EntityInteractionRange,
    ExplosionKnockbackResistance,
    FallDamageMultiplier,
    FlyingSpeed,
    FollowRange,
    Gravity,
    JumpStrength,
    KnockbackResistance,
    Luck,
    MaxAbsorption,
    MaxHealth,
    MiningEfficiency,
    MovementEfficiency,
    MovementSpeed,
    OxygenBonus,
    SafeFallDistance,
    Scale,
    SneakingSpeed,
    SpawnReinforcements,
    StepHeight,
    SubmergedMiningSpeed,
    SweepingDamageRatio,
    TemptRange,
    WaterMovementEfficiency,
}

impl Attribute {
    pub(crate) fn to_java_attribute(&self) -> Result<JavaObject> {
        let mut env = get_env()?;

        // Get the Attribute class
        let attribute_class = env.find_class("net/minestom/server/entity/attribute/Attribute")?;

        // Get the static attribute field
        let field_name = self.to_field_name();
        let attribute = env
            .get_static_field(
                attribute_class,
                field_name,
                "Lnet/minestom/server/entity/attribute/Attribute;",
            )?
            .l()?;

        Ok(JavaObject::from_env(&mut env, attribute)?)
    }

    fn to_field_name(&self) -> &'static str {
        match self {
            Attribute::Armor => "ARMOR",
            Attribute::ArmorToughness => "ARMOR_TOUGHNESS",
            Attribute::AttackDamage => "ATTACK_DAMAGE",
            Attribute::AttackKnockback => "ATTACK_KNOCKBACK",
            Attribute::AttackSpeed => "ATTACK_SPEED",
            Attribute::BlockBreakSpeed => "BLOCK_BREAK_SPEED",
            Attribute::BlockInteractionRange => "BLOCK_INTERACTION_RANGE",
            Attribute::BurningTime => "BURNING_TIME",
            Attribute::EntityInteractionRange => "ENTITY_INTERACTION_RANGE",
            Attribute::ExplosionKnockbackResistance => "EXPLOSION_KNOCKBACK_RESISTANCE",
            Attribute::FallDamageMultiplier => "FALL_DAMAGE_MULTIPLIER",
            Attribute::FlyingSpeed => "FLYING_SPEED",
            Attribute::FollowRange => "FOLLOW_RANGE",
            Attribute::Gravity => "GRAVITY",
            Attribute::JumpStrength => "JUMP_STRENGTH",
            Attribute::KnockbackResistance => "KNOCKBACK_RESISTANCE",
            Attribute::Luck => "LUCK",
            Attribute::MaxAbsorption => "MAX_ABSORPTION",
            Attribute::MaxHealth => "MAX_HEALTH",
            Attribute::MiningEfficiency => "MINING_EFFICIENCY",
            Attribute::MovementEfficiency => "MOVEMENT_EFFICIENCY",
            Attribute::MovementSpeed => "MOVEMENT_SPEED",
            Attribute::OxygenBonus => "OXYGEN_BONUS",
            Attribute::SafeFallDistance => "SAFE_FALL_DISTANCE",
            Attribute::Scale => "SCALE",
            Attribute::SneakingSpeed => "SNEAKING_SPEED",
            Attribute::SpawnReinforcements => "SPAWN_REINFORCEMENTS",
            Attribute::StepHeight => "STEP_HEIGHT",
            Attribute::SubmergedMiningSpeed => "SUBMERGED_MINING_SPEED",
            Attribute::SweepingDamageRatio => "SWEEPING_DAMAGE_RATIO",
            Attribute::TemptRange => "TEMPT_RANGE",
            Attribute::WaterMovementEfficiency => "WATER_MOVEMENT_EFFICIENCY",
        }
    }
}

/// Represents an attribute instance that can be modified
pub struct AttributeInstance {
    inner: JavaObject,
}

impl AttributeInstance {
    pub(crate) fn new(inner: JavaObject) -> Self {
        Self { inner }
    }

    /// Gets the base value of the attribute
    pub fn base_value(&self) -> Result<f64> {
        self.inner.call_double_method("getBaseValue", "()D", &[])
    }

    /// Sets the base value of the attribute
    pub fn set_base_value(&self, value: f64) -> Result<()> {
        self.inner.call_void_method(
            "setBaseValue",
            "(D)V",
            &[value.into()],
        )
    }
} 