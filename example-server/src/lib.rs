use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::jint;
use minestom_rs::jni_utils;
use std::panic;

mod lobby;
mod parkour;

#[unsafe(no_mangle)]
pub extern "system" fn Java_org_example_Main_startServer(env: JNIEnv, class: JClass) -> jint {
    // Attach JVM to the current thread
    let res = jni_utils::attach_jvm(&env);

    match res {
        Ok(_) => (),
        Err(e) => {
            eprintln!("failed to attach JVM: {}", e);
            return -1;
        }
    }

    // Build tokio runtime
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("failed to build tokio runtime: {}", e);
            return -1;
        }
    };

    // Run server
    let result = panic::catch_unwind(|| {
        runtime.block_on(async {
            match parkour::run_server().await {
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
