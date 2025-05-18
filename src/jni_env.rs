use crate::error::MinestomError;
use crate::Result;
use jni::objects::JString;
use jni::{JNIEnv, JavaVM};
use std::cell::RefCell;

thread_local! {
    static THREAD_ENV: RefCell<Option<ThreadEnvGuard>> = RefCell::new(None);
}

pub struct ThreadEnvGuard {
    env: *mut jni::sys::JNIEnv,
    _guard: jni::AttachGuard<'static>,
}

// SAFETY: ThreadEnvGuard is only accessed from one thread
unsafe impl Send for ThreadEnvGuard {}
unsafe impl Sync for ThreadEnvGuard {}

impl Drop for ThreadEnvGuard {
    fn drop(&mut self) {
        // The AttachGuard will handle detaching the thread when dropped
    }
}

/// Check if there's a Java exception and convert it to a Rust error
pub fn check_exception(env: &mut JNIEnv) -> Result<()> {
    if env.exception_check()? {
        let exception = env.exception_occurred()?;
        env.exception_clear()?;

        let msg = if let Ok(msg) =
            env.call_method(exception, "getMessage", "()Ljava/lang/String;", &[])
        {
            if let Ok(msg) = msg.l() {
                if let Ok(jstr) = env.get_string(&JString::from(msg)) {
                    jstr.to_string_lossy().into_owned()
                } else {
                    "Unknown error".to_string()
                }
            } else {
                "Unknown error".to_string()
            }
        } else {
            "Unknown error".to_string()
        };

        Err(MinestomError::EventError(msg))
    } else {
        Ok(())
    }
}
