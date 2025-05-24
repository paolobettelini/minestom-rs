use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::jint;
use minestom_rs::jni_utils;
use minestom_rs::{RUNTIME, init_runtime};
use once_cell::sync::Lazy;
use std::panic;
use tokio::runtime::{Builder, Handle};

mod commands;
mod favicon;
mod magic_values;
mod maps;
mod mojang;
mod logic;
mod server;

#[unsafe(no_mangle)]
pub extern "system" fn Java_org_example_Main_startServer(env: JNIEnv, class: JClass) -> jint {
    init_runtime();

    // Attach JVM to the current thread
    let res = jni_utils::attach_jvm(&env);

    match res {
        Ok(_) => (),
        Err(e) => {
            eprintln!("failed to attach JVM: {}", e);
            return -1;
        }
    }

    let result = panic::catch_unwind(|| {
        // RUNTIME is the Lazy<Runtime> from your library
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
