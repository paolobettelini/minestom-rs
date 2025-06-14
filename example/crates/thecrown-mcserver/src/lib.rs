use jni::{JNIEnv, objects::JClass, sys::jint};
use thecrown_common as common;

mod commands;
mod lobby;
mod magic_values;
mod maps;
mod server;

#[unsafe(no_mangle)]
pub extern "system" fn Java_net_thecrown_App_startServer(env: JNIEnv, _class: JClass) -> jint {
    common::jni::jni_entry(&env, || async {
        match server::run_server().await {
            Ok(_) => 0,
            Err(e) => {
                eprintln!("server error: {}", e);
                -1
            }
        }
    })
}
