use crate::Result;
use crate::event::EventNode;
use crate::jni_utils::{JavaObject, JniValue, get_env};
use jni::JNIEnv;
use log::{debug, error};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

// Store task callbacks
static TASK_CALLBACKS: Lazy<RwLock<HashMap<u64, Arc<dyn Fn() -> Result<()> + Send + Sync>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

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

        debug!(
            "[RUST-SCHEDULER] Creating task executor with id: {}",
            callback_id
        );

        // Create the task executor
        let executor_class = match env.find_class("rust/minestom/TaskExecutorCallback") {
            Ok(class) => {
                debug!("[RUST-SCHEDULER] Found TaskExecutorCallback class");
                class
            }
            Err(e) => {
                error!(
                    "[RUST-SCHEDULER] Failed to find TaskExecutorCallback class: {}",
                    e
                );
                if let Ok(exception) = env.exception_occurred() {
                    if !exception.is_null() {
                        env.exception_describe()?;
                        env.exception_clear()?;
                    }
                }
                return Err(e.into());
            }
        };

        let executor = match env.new_object(
            executor_class,
            "(J)V",
            &[JniValue::Long(callback_id as i64).as_jvalue()],
        ) {
            Ok(obj) => {
                debug!("[RUST-SCHEDULER] Created TaskExecutorCallback instance");
                obj
            }
            Err(e) => {
                error!(
                    "[RUST-SCHEDULER] Failed to create TaskExecutorCallback instance: {}",
                    e
                );
                if let Ok(exception) = env.exception_occurred() {
                    if !exception.is_null() {
                        env.exception_describe()?;
                        env.exception_clear()?;
                    }
                }
                return Err(e.into());
            }
        };

        let executor_global = env.new_global_ref(executor)?;
        debug!("[RUST-SCHEDULER] Created global reference for executor");

        // Get the ExecutionType.SYNC enum value
        let execution_type_class = match env.find_class("net/minestom/server/timer/ExecutionType") {
            Ok(class) => {
                debug!("[RUST-SCHEDULER] Found ExecutionType class");
                class
            }
            Err(e) => {
                error!("[RUST-SCHEDULER] Failed to find ExecutionType class: {}", e);
                if let Ok(exception) = env.exception_occurred() {
                    if !exception.is_null() {
                        env.exception_describe()?;
                        env.exception_clear()?;
                    }
                }
                return Err(e.into());
            }
        };

        let sync_type = match env.get_static_field(
            execution_type_class,
            "TICK_START",
            "Lnet/minestom/server/timer/ExecutionType;",
        ) {
            Ok(field) => {
                debug!("[RUST-SCHEDULER] Got TICK_START execution type");
                field.l()?
            }
            Err(e) => {
                error!(
                    "[RUST-SCHEDULER] Failed to get TICK_START execution type: {}",
                    e
                );
                if let Ok(exception) = env.exception_occurred() {
                    if !exception.is_null() {
                        env.exception_describe()?;
                        env.exception_clear()?;
                    }
                }
                return Err(e.into());
            }
        };

        // Create a TaskSchedule supplier
        let supplier_class = match env.find_class("rust/minestom/TaskScheduleSupplier") {
            Ok(class) => {
                debug!("[RUST-SCHEDULER] Found TaskScheduleSupplier class");
                class
            }
            Err(e) => {
                error!(
                    "[RUST-SCHEDULER] Failed to find TaskScheduleSupplier class: {}",
                    e
                );
                if let Ok(exception) = env.exception_occurred() {
                    if !exception.is_null() {
                        env.exception_describe()?;
                        env.exception_clear()?;
                    }
                }
                return Err(e.into());
            }
        };

        let supplier = match env.new_object(
            supplier_class,
            "(Ljava/lang/Runnable;)V",
            &[JniValue::Object(JavaObject::global_to_local(&executor_global)?).as_jvalue()],
        ) {
            Ok(obj) => {
                debug!("[RUST-SCHEDULER] Created TaskScheduleSupplier instance");
                obj
            }
            Err(e) => {
                error!(
                    "[RUST-SCHEDULER] Failed to create TaskScheduleSupplier instance: {}",
                    e
                );
                if let Ok(exception) = env.exception_occurred() {
                    if !exception.is_null() {
                        env.exception_describe()?;
                        env.exception_clear()?;
                    }
                }
                return Err(e.into());
            }
        };

        let supplier_global = env.new_global_ref(supplier)?;
        debug!("[RUST-SCHEDULER] Created global reference for supplier");

        // Submit the task
        debug!("[RUST-SCHEDULER] Submitting task to scheduler");
        let task = match self.inner.call_object_method(
            "submitTask",
            "(Ljava/util/function/Supplier;Lnet/minestom/server/timer/ExecutionType;)Lnet/minestom/server/timer/Task;",
            &[
                JniValue::Object(JavaObject::global_to_local(&supplier_global)?),
                JniValue::Object(env.new_local_ref(sync_type)?),
            ],
        ) {
            Ok(task) => {
                debug!("[RUST-SCHEDULER] Task submitted successfully");
                task
            },
            Err(e) => {
                error!("[RUST-SCHEDULER] Failed to submit task: {}", e);
                if let Ok(exception) = env.exception_occurred() {
                    if !exception.is_null() {
                        env.exception_describe()?;
                        env.exception_clear()?;
                    }
                }
                return Err(e.into());
            }
        };

        let task_builder = TaskBuilder::new(task);
        task_builder.repeat(1)?; // Make the task repeat every tick by default
        Ok(task_builder)
    }
}

impl TaskBuilder {
    pub(crate) fn new(inner: JavaObject) -> Self {
        Self { inner }
    }

    /// Schedules the task to be executed
    pub fn schedule(&self) -> Result<()> {
        // Task is already scheduled when created by submitTask
        Ok(())
    }

    /// Sets the delay before the task starts executing
    pub fn delay(&self, ticks: i64) -> Result<&Self> {
        let mut env = get_env()?;

        // First get the TaskScheduleSupplier from the task
        let supplier = self.inner.call_object_method(
            "getScheduler",
            "()Ljava/util/function/Supplier;",
            &[],
        )?;

        // Call setDelay on the supplier
        supplier.call_object_method("setDelay", "(J)V", &[JniValue::Long(ticks)])?;

        Ok(self)
    }

    /// Sets the task to repeat with the given interval
    pub fn repeat(&self, ticks: i64) -> Result<&Self> {
        let mut env = get_env()?;

        // First get the TaskScheduleSupplier from the task
        let supplier = self.inner.call_object_method(
            "getScheduler",
            "()Ljava/util/function/Supplier;",
            &[],
        )?;

        // Call setRepeat on the supplier
        supplier.call_void_method("setRepeat", "(J)V", &[JniValue::Long(ticks)])?;

        Ok(self)
    }

    /// Sets whether the task should be executed iteratively
    pub fn iterative(&self, iterative: bool) -> Result<&Self> {
        let mut env = get_env()?;
        self.inner.call_object_method(
            "iterative",
            "(Z)Lnet/minestom/server/timer/Task;",
            &[JniValue::Bool(iterative)],
        )?;
        Ok(self)
    }

    pub fn event_node(&self) -> Result<EventNode> {
        let mut env = get_env()?;
        let event_node = self.inner.call_object_method(
            "eventNode",
            "()Lnet/minestom/server/event/EventNode;",
            &[],
        )?;
        Ok(EventNode::from(event_node))
    }
}

impl Drop for TaskBuilder {
    fn drop(&mut self) {
        // Clean up any global references when the task is dropped
        if let Ok(mut env) = get_env() {
            if let Ok(exception) = env.exception_occurred() {
                if !exception.is_null() {
                    let _ = env.exception_clear();
                }
            }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_rust_minestom_TaskExecutorCallback_executeTask(
    env: *mut jni::sys::JNIEnv,
    _this: jni::objects::JObject,
    callback_id: jni::sys::jlong,
) {
    eprintln!(
        "[DEBUG-RUST] Starting executeTask with callback_id: {}",
        callback_id
    );

    // Get project root directory for logging
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    eprintln!("[DEBUG-RUST] Current directory: {:?}", current_dir);
    let project_root = current_dir.parent().unwrap_or(&current_dir).to_path_buf();
    eprintln!("[DEBUG-RUST] Project root: {:?}", project_root);
    let logs_dir = project_root.join("logs");
    eprintln!("[DEBUG-RUST] Creating logs directory at: {:?}", logs_dir);
    match std::fs::create_dir_all(&logs_dir) {
        Ok(_) => eprintln!("[DEBUG-RUST] Successfully created/verified logs directory"),
        Err(e) => eprintln!("[DEBUG-RUST] Error creating logs directory: {}", e),
    }

    let log_path = logs_dir.join("error_log.txt");
    eprintln!("[DEBUG-RUST] Opening log file at: {:?}", log_path);

    let mut log_file = match OpenOptions::new().create(true).append(true).open(&log_path) {
        Ok(file) => {
            eprintln!("[DEBUG-RUST] Successfully opened log file");
            file
        }
        Err(e) => {
            eprintln!(
                "[DEBUG-RUST] Failed to open log file at {:?}: {}",
                log_path, e
            );
            return;
        }
    };

    writeln!(log_file, "\n=== RUST TASK EXECUTION START ===")
        .unwrap_or_else(|e| eprintln!("[DEBUG-RUST] Error writing to log file: {}", e));
    writeln!(
        log_file,
        "[RUST] Starting task execution for callback id: {}",
        callback_id
    )
    .unwrap_or_else(|e| eprintln!("[DEBUG-RUST] Error writing to log file: {}", e));

    // Update panic hook to use the same logs directory
    let logs_dir_clone = logs_dir.clone();
    std::panic::set_hook(Box::new(move |panic_info| {
        let log_path = logs_dir_clone.join("error_log.txt");
        let mut log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .unwrap_or_else(|e| panic!("Failed to open log file: {}", e));

        writeln!(log_file, "\n!!! RUST PANIC !!!").unwrap_or_default();
        writeln!(log_file, "Panic info: {:?}", panic_info).unwrap_or_default();
        if let Some(location) = panic_info.location() {
            writeln!(
                log_file,
                "Panic occurred in file '{}' at line {}",
                location.file(),
                location.line()
            )
            .unwrap_or_default();
        }
        writeln!(
            log_file,
            "Backtrace:\n{:?}",
            std::backtrace::Backtrace::capture()
        )
        .unwrap_or_default();
    }));

    // Convert the raw JNIEnv pointer to a safe wrapper
    let mut env = match unsafe { JNIEnv::from_raw(env) } {
        Ok(env) => env,
        Err(e) => {
            writeln!(log_file, "[RUST-ERROR] Failed to get JNIEnv: {}", e).unwrap_or_default();
            writeln!(
                log_file,
                "Backtrace:\n{:?}",
                std::backtrace::Backtrace::capture()
            )
            .unwrap_or_default();
            return;
        }
    };

    // Create a frame to manage local references
    let _frame = match env.push_local_frame(16) {
        Ok(frame) => frame,
        Err(e) => {
            writeln!(log_file, "[RUST-ERROR] Failed to create local frame: {}", e)
                .unwrap_or_default();
            writeln!(
                log_file,
                "Backtrace:\n{:?}",
                std::backtrace::Backtrace::capture()
            )
            .unwrap_or_default();
            return;
        }
    };

    // Get the callback from our global map
    let callback = {
        let callbacks = TASK_CALLBACKS.read();
        match callbacks.get(&(callback_id as u64)) {
            Some(callback) => callback.clone(),
            None => {
                writeln!(
                    log_file,
                    "[RUST-ERROR] No callback found for id: {}",
                    callback_id
                )
                .unwrap_or_default();
                writeln!(
                    log_file,
                    "Available callbacks: {:?}",
                    callbacks.keys().collect::<Vec<_>>()
                )
                .unwrap_or_default();
                return;
            }
        }
    };

    // Execute the callback
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| callback())) {
        Ok(result) => {
            match result {
                Ok(_) => {
                    writeln!(log_file, "[RUST] Task executed successfully").unwrap_or_default();
                }
                Err(e) => {
                    writeln!(log_file, "\n!!! RUST TASK ERROR !!!").unwrap_or_default();
                    writeln!(log_file, "[RUST-ERROR] Error executing task: {}", e)
                        .unwrap_or_default();
                    writeln!(
                        log_file,
                        "Backtrace:\n{:?}",
                        std::backtrace::Backtrace::capture()
                    )
                    .unwrap_or_default();

                    // First check for any pending exception
                    if let Ok(exception) = env.exception_occurred() {
                        if !exception.is_null() {
                            writeln!(log_file, "\n=== JAVA EXCEPTION DETAILS (from Rust) ===")
                                .unwrap_or_default();

                            // Force print the exception immediately
                            let _ = env.exception_describe();

                            // Get the exception class and name
                            if let Ok(exception_class) = env.get_object_class(&exception) {
                                if let Ok(class_name) = env.call_method(
                                    exception_class,
                                    "getName",
                                    "()Ljava/lang/String;",
                                    &[],
                                ) {
                                    if let Ok(class_name_obj) = class_name.l() {
                                        if let Ok(class_name_str) =
                                            env.get_string(&class_name_obj.into())
                                        {
                                            writeln!(
                                                log_file,
                                                "Exception class: {}",
                                                class_name_str.to_string_lossy()
                                            )
                                            .unwrap_or_default();
                                        }
                                    }
                                }
                            }

                            // Get the exception message
                            if let Ok(msg) = env.call_method(
                                &exception,
                                "getMessage",
                                "()Ljava/lang/String;",
                                &[],
                            ) {
                                if let Ok(msg_obj) = msg.l() {
                                    if let Ok(msg_str) = env.get_string(&msg_obj.into()) {
                                        writeln!(
                                            log_file,
                                            "Exception message: {}",
                                            msg_str.to_string_lossy()
                                        )
                                        .unwrap_or_default();
                                    }
                                }
                            }

                            // Get and print the stack trace
                            if let Ok(stack_trace) = env.call_method(
                                &exception,
                                "getStackTrace",
                                "()[Ljava/lang/StackTraceElement;",
                                &[],
                            ) {
                                if let Ok(stack_trace_array) = stack_trace.l() {
                                    let array = unsafe {
                                        jni::objects::JObjectArray::from_raw(
                                            stack_trace_array.as_raw(),
                                        )
                                    };
                                    if let Ok(length) = env.get_array_length(&array) {
                                        writeln!(log_file, "\nStack trace:").unwrap_or_default();
                                        for i in 0..length {
                                            if let Ok(element) =
                                                env.get_object_array_element(&array, i)
                                            {
                                                if let Ok(str_result) = env.call_method(
                                                    element,
                                                    "toString",
                                                    "()Ljava/lang/String;",
                                                    &[],
                                                ) {
                                                    if let Ok(str_obj) = str_result.l() {
                                                        if let Ok(str_val) =
                                                            env.get_string(&str_obj.into())
                                                        {
                                                            writeln!(
                                                                log_file,
                                                                "    at {}",
                                                                str_val.to_string_lossy()
                                                            )
                                                            .unwrap_or_default();
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            writeln!(log_file, "=== END JAVA EXCEPTION DETAILS ===\n")
                                .unwrap_or_default();

                            // Clear the exception after we've printed all details
                            let _ = env.exception_clear();

                            // Throw a new exception with our gathered details
                            if let Ok(exception_class) =
                                env.find_class("java/lang/RuntimeException")
                            {
                                let message = format!("Task execution failed - Error: {}", e);
                                let _ = env.throw_new(exception_class, &message);
                            }
                        } else {
                            writeln!(log_file, "[RUST-ERROR] No Java exception found, but task failed with error: {}", e).unwrap_or_default();
                        }
                    } else {
                        writeln!(log_file, "[RUST-ERROR] Failed to check for Java exception")
                            .unwrap_or_default();
                    }
                }
            }
        }
        Err(e) => {
            writeln!(log_file, "\n!!! RUST PANIC IN CALLBACK !!!").unwrap_or_default();
            writeln!(
                log_file,
                "[RUST-ERROR] Panic occurred in task callback: {:?}",
                e
            )
            .unwrap_or_default();
            writeln!(
                log_file,
                "Backtrace:\n{:?}",
                std::backtrace::Backtrace::capture()
            )
            .unwrap_or_default();

            // Create a RuntimeException with the panic message
            if let Ok(exception_class) = env.find_class("java/lang/RuntimeException") {
                let message = format!("Rust panic in task {}: {:?}", callback_id, e);
                let _ = env.throw_new(exception_class, &message);
            }
        }
    }

    // Clean up the callback after execution
    TASK_CALLBACKS.write().remove(&(callback_id as u64));
    writeln!(
        log_file,
        "[RUST] Task cleanup completed for callback id: {}",
        callback_id
    )
    .unwrap_or_default();
    writeln!(log_file, "=== RUST TASK EXECUTION END ===\n").unwrap_or_default();

    // The frame will be automatically popped when _frame is dropped
}
