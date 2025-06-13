use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::jint;
use minestom::jni_utils;
use minestom::{RUNTIME, init_runtime};
use std::panic;
use thecrown_common as common;

mod server;
mod favicon;

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
