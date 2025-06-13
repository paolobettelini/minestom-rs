use jni::JNIEnv;
use jni::objects::{JObject, JValue};

use crate::entity::SendablePacket;
use crate::jni_utils::get_env;

pub struct TransferPacket {
    pub host: String,
    pub port: u16,
}

impl SendablePacket for TransferPacket {
    fn to_java(&self) -> JObject {
        let mut env = get_env().unwrap();
        // Crea Java String per l'host
        let java_host = env
            .new_string(&self.host)
            .expect("Failed to create Java String for host");
        // Trova la classe TransferPacket
        let pkt_cls = env
            .find_class("net/minestom/server/network/packet/server/common/TransferPacket")
            .expect("Failed to find TransferPacket class");
        // Costruisci TransferPacket(host: String, port: int)
        let pkt_obj = env
            .new_object(
                pkt_cls,
                "(Ljava/lang/String;I)V",
                &[
                    JValue::Object(&JObject::from(java_host)),
                    JValue::Int(self.port.into()),
                ],
            )
            .expect("Failed to construct Java TransferPacket");
        pkt_obj
    }
}

impl TransferPacket {
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }
}
