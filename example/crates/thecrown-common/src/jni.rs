use jni::JNIEnv;
use jni::sys::jint;
use minestom::jni_utils;
use minestom::{RUNTIME, init_runtime};
use std::future::Future;
use std::panic;

/// A generic JNI entrypoint wrapper.  
/// F: a zero‑arg closure returning a Future whose Output is jint  
/// (so you can map your Result<…,E> → 0/–1 yourself inside the closure).
pub fn jni_entry<F, Fut>(env: &JNIEnv, f: F) -> jint
where
    F: FnOnce() -> Fut + panic::UnwindSafe,
    Fut: Future<Output = jint>,
{
    // ensure our async runtime is initialized
    init_runtime();

    // attach this thread to the JVM
    if let Err(e) = jni_utils::attach_jvm(env) {
        eprintln!("failed to attach JVM: {}", e);
        return -1;
    }

    // catch panics inside your async block
    let result = panic::catch_unwind(|| RUNTIME.block_on(f()));

    match result {
        Ok(code) => code,
        Err(_) => {
            eprintln!("panic in Rust server");
            -1
        }
    }
}
