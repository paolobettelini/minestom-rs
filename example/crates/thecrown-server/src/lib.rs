use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::jint;
use minestom::jni_utils;
use minestom::{RUNTIME, init_runtime};
use once_cell::sync::Lazy;
use std::panic;
use tokio::runtime::{Builder, Handle};

mod commands;
mod favicon;
mod logic;
mod magic_values;
mod maps;
mod mojang;
mod server;

#[unsafe(no_mangle)]
pub extern "system" fn Java_net_thecrown_App_startServer(env: JNIEnv, class: JClass) -> jint {
    // Attach JVM to the current thread
    init_runtime();
    let res = jni_utils::attach_jvm(&env);

    match res {
        Ok(_) => (),
        Err(e) => {
            eprintln!("failed to attach JVM: {}", e);
            return -1;
        }
    }

    let result = panic::catch_unwind(|| {
        RUNTIME.block_on(async {
            match server::run_server().await {
                Ok(_) => 0,
                Err(e) => {
                    eprintln!("server error: {}", e);
                    -1
                }
            }
        })
    });

    match result {
        Ok(code) => code,
        Err(_) => {
            eprintln!("panic in Rust server");
            -1
        }
    }
}
