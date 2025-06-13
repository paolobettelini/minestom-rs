use jni::JNIEnv;
use jni::objects::{JObject, JString, JValue};

use crate::entity::SendablePacket;
use crate::jni_utils::get_env;

pub struct StoreCookiePacket<'a> {
    pub key: &'a str,
    pub data: Vec<u8>,
}

impl<'a> SendablePacket for StoreCookiePacket<'a> {
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
        // Trova la classe StoreCookiePacket
        let pkt_cls = env
            .find_class("net/minestom/server/network/packet/server/common/StoreCookiePacket")
            .expect("Failed to find StoreCookiePacket class");
        // Costruisci StoreCookiePacket(key: String, data: byte[])
        let pkt_obj = env
            .new_object(
                pkt_cls,
                "(Ljava/lang/String;[B)V",
                &[
                    JValue::Object(&JObject::from(java_key)),
                    JValue::Object(&JObject::from(java_data)),
                ],
            )
            .expect("Failed to construct Java StoreCookiePacket");
        pkt_obj
    }
}

impl<'a> StoreCookiePacket<'a> {
    pub fn new(key: &'a str, data: Vec<u8>) -> Self {
        Self { key, data }
    }
}

