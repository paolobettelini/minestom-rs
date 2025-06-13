use jni::JNIEnv;
use jni::objects::{JObject, JValue};

use crate::entity::SendablePacket;
use crate::jni_utils::get_env;

/// Packet per memorizzazione cookie (CookieStorePacket)
pub struct CookieStorePacket<'a> {
    pub key: &'a str,
    pub data: Vec<u8>,
}

impl<'a> SendablePacket for CookieStorePacket<'a> {
    /// Converte in oggetto Java CookieStorePacket
    fn to_java(&self) -> JObject {
        let mut env = get_env().unwrap();
        // Crea Java String per la chiave
        let java_key = env
            .new_string(&self.key)
            .expect("Failed to create Java String for key");
        // Crea array di byte per i dati
        let java_data = env
            .byte_array_from_slice(&self.data)
            .expect("Failed to create Java byte[] from Rust Vec<u8>");
        // Trova la classe CookieStorePacket
        let pkt_cls = env
            .find_class("net/minestom/server/network/packet/server/common/CookieStorePacket")
            .expect("Failed to find CookieStorePacket class");
        // Costruisci CookieStorePacket(key: String, data: byte[])
        let pkt_obj = env
            .new_object(
                pkt_cls,
                "(Ljava/lang/String;[B)V",
                &[
                    JValue::Object(&JObject::from(java_key)),
                    JValue::Object(&JObject::from(java_data)),
                ],
            )
            .expect("Failed to construct Java CookieStorePacket");
        pkt_obj
    }
}

impl<'a> CookieStorePacket<'a> {
    /// Crea un nuovo CookieStorePacket con chiave e dati
    pub fn new(key: &'a str, data: Vec<u8>) -> Self {
        Self { key, data }
    }
}
