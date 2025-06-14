use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::jint;
use thecrown_common as common;

mod advancements;
mod commands;
mod logic;
mod magic_values;
mod maps;
mod models;
mod mojang;
mod server;

#[unsafe(no_mangle)]
pub extern "system" fn Java_net_thecrown_App_startServer(env: JNIEnv, class: JClass) -> jint {
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
