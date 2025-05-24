use crate::Result;
use crate::jni_utils::{JavaObject, JniValue, get_env};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

// Store task callbacks
static TASK_CALLBACKS: Lazy<
    RwLock<HashMap<u64, Arc<dyn Fn() -> Result<()> + Send + Sync>>>,
> = Lazy::new(|| RwLock::new(HashMap::new()));

static NEXT_CALLBACK_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub struct SchedulerManager {
    inner: JavaObject,
}

pub struct TaskBuilder {
    inner: JavaObject,
}

impl SchedulerManager {
    pub(crate) fn new(inner: JavaObject) -> Self {
        Self { inner }
    }

    /// Creates a new task builder for scheduling tasks
    pub fn build_task<F>(&self, task: F) -> Result<TaskBuilder>
    where
        F: Fn() -> Result<()> + Send + Sync + 'static,
    {
        let mut env = get_env()?;

        // Store the task callback
        let callback_id = NEXT_CALLBACK_ID.fetch_add(1, Ordering::SeqCst);
        let callback = Arc::new(task);
        TASK_CALLBACKS.write().insert(callback_id, callback);

        // Create the task executor
        let executor_class = env.find_class("org/example/TaskExecutorCallback")?;
        let executor = env.new_object(
            executor_class,
            "(J)V",
            &[JniValue::Long(callback_id as i64).as_jvalue()],
        )?;

        // Build the task
        let task = self.inner.call_object_method(
            "buildTask",
            "(Ljava/lang/Runnable;)Lnet/minestom/server/timer/TaskSchedule;",
            &[JniValue::Object(executor)],
        )?;

        Ok(TaskBuilder::new(task))
    }
}

impl TaskBuilder {
    pub(crate) fn new(inner: JavaObject) -> Self {
        Self { inner }
    }

    /// Schedules the task to be executed
    pub fn schedule(&self) -> Result<()> {
        self.inner.call_void_method("schedule", "()V", &[])?;
        Ok(())
    }

    /// Sets the delay before the task starts executing
    pub fn delay(&self, ticks: i64) -> Result<&Self> {
        self.inner.call_void_method(
            "delay",
            "(J)Lnet/minestom/server/timer/TaskSchedule;",
            &[JniValue::Long(ticks)],
        )?;
        Ok(self)
    }

    /// Sets the task to repeat with the given interval
    pub fn repeat(&self, ticks: i64) -> Result<&Self> {
        self.inner.call_void_method(
            "repeat",
            "(J)Lnet/minestom/server/timer/TaskSchedule;",
            &[JniValue::Long(ticks)],
        )?;
        Ok(self)
    }

    /// Sets whether the task should be executed iteratively
    pub fn iterative(&self, iterative: bool) -> Result<&Self> {
        self.inner.call_void_method(
            "iterative",
            "(Z)Lnet/minestom/server/timer/TaskSchedule;",
            &[JniValue::Bool(iterative)],
        )?;
        Ok(self)
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_org_example_TaskExecutorCallback_executeTask(
    env: *mut jni::sys::JNIEnv,
    _class: jni::objects::JClass,
    callback_id: jni::sys::jlong,
) {
    // Catch any panic to prevent unwinding into Java
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // Get the callback from our global map
        let callback = {
            let callbacks = TASK_CALLBACKS.read();
            match callbacks.get(&(callback_id as u64)) {
                Some(callback) => callback.clone(),
                None => {
                    log::error!("No callback found for id: {}", callback_id);
                    return;
                }
            }
        };

        // Execute the callback
        if let Err(e) = callback() {
            log::error!("Error executing task: {}", e);
        }
    }));

    if let Err(e) = result {
        log::error!("Panic occurred in task callback: {:?}", e);
    }
} 