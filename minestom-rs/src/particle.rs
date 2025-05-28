use jni::JNIEnv;
use jni::objects::{JObject, JValue};

use crate::entity::SendablePacket;
use crate::jni_utils::get_env;

/// All built-in Minestom particles. Extend with new variants as needed.
#[derive(Debug, Clone, Copy)]
pub enum ParticleType {
    Block,
    Critical,
    Smoke,
    Flame,
    Heart,
    Note,
    // ... altri tipi da net/minestom/server/particle/Particle
}

impl ParticleType {
    /// Nome del campo statico Java corrispondente
    pub fn to_java_field(&self) -> &'static str {
        match self {
            ParticleType::Block => "BLOCK",
            ParticleType::Critical => "CRITICAL",
            ParticleType::Smoke => "SMOKE",
            ParticleType::Flame => "FLAME",
            ParticleType::Heart => "HEART",
            ParticleType::Note => "NOTE",
        }
    }
}

/// Packet per spawn di particelle
pub struct ParticlePacket {
    pub particle: ParticleType,
    pub override_limiter: bool,
    pub long_distance: bool,
    pub position: (f64, f64, f64),
    pub offset: (f32, f32, f32),
    pub max_speed: f32,
    pub count: i32,
}

impl SendablePacket for ParticlePacket {
    /// Converte in oggetto Java ParticlePacket
    fn to_java(&self) -> JObject {
        let mut env = get_env().unwrap();
        // Recupera campo statico Particle
        let cls_particle = env
            .find_class("net/minestom/server/particle/Particle")
            .unwrap();
        let field_name = self.particle.to_java_field();
        let java_particle = env
            .get_static_field(
                cls_particle,
                field_name,
                "Lnet/minestom/server/particle/Particle;",
            )
            .unwrap()
            .l()
            .unwrap();
        // Costruisci ParticlePacket
        let pkt_cls = env
            .find_class("net/minestom/server/network/packet/server/play/ParticlePacket")
            .unwrap();
        let pkt_obj = env
            .new_object(
                pkt_cls,
                "(Lnet/minestom/server/particle/Particle;ZZDDDFFFFI)V",
                &[
                    JValue::Object((&java_particle).into()),
                    JValue::Bool(self.override_limiter as u8),
                    JValue::Bool(self.long_distance as u8),
                    JValue::Double(self.position.0),
                    JValue::Double(self.position.1),
                    JValue::Double(self.position.2),
                    JValue::Float(self.offset.0),
                    JValue::Float(self.offset.1),
                    JValue::Float(self.offset.2),
                    JValue::Float(self.max_speed),
                    JValue::Int(self.count),
                ],
            )
            .unwrap();
        pkt_obj
    }
}

impl ParticlePacket {
    pub fn new(particle: ParticleType, x: f64, y: f64, z: f64) -> Self {
        Self {
            particle,
            override_limiter: false,
            long_distance: false,
            position: (x, y, z),
            offset: (0.0, 0.0, 0.0),
            max_speed: 0.0,
            count: 1,
        }
    }
}
