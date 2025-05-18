use crate::error::MinestomError;
use crate::Result;
use jni::objects::{GlobalRef, JObject, JString, JValueGen};
use jni::{JNIEnv, JavaVM};
use log::debug;
use parking_lot::Mutex;
use std::cell::RefCell;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread_local;

lazy_static::lazy_static! {
    static ref JVM_INSTANCE: Arc<Mutex<Option<Arc<JavaVM>>>> = Arc::new(Mutex::new(None));
    static ref JVM_INITIALIZED: AtomicBool = AtomicBool::new(false);
}

thread_local! {
    static THREAD_ENV: RefCell<Option<ThreadEnvGuard>> = RefCell::new(None);
}

pub(crate) struct ThreadEnvGuard {
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

/// Attaches to the JVM from an existing JNIEnv.
/// This should be called (once) in JNI_OnLoad or first native call.
pub fn attach_jvm(env: &JNIEnv) -> Result<()> {
    // If already initialized, skip
    if JVM_INITIALIZED.load(Ordering::SeqCst) {
        log::debug!("JVM already attached");
        return Ok(());
    }

    // Retrieve the raw JavaVM handle from the provided JNIEnv
    let raw_vm: JavaVM = env
        .get_java_vm()
        .map_err(|e| MinestomError::JvmInit(format!("Failed to get JavaVM from JNIEnv: {}", e)))?;
    let arc_vm = Arc::new(raw_vm);

    // Store the Arc<JavaVM> into our static
    {
        let mut jvm_guard = JVM_INSTANCE.lock();
        *jvm_guard = Some(arc_vm.clone());
    }

    // Mark as initialized
    JVM_INITIALIZED.store(true, Ordering::SeqCst);
    log::info!("JVM attached successfully");

    // Quick basic verification to ensure class loading works
    // let mut verify_env = arc_vm.attach_current_thread()
    //     .map_err(|e| MinestomError::JvmInit(format!("Failed to attach thread for verification: {}", e)))?;
    // verify_env.find_class("java/lang/String")
    //     .map_err(|e| MinestomError::JvmInit(format!("JVM verification failed: {}", e)))?;
    // log::info!("JVM verification successful");

    Ok(())
}

/// Gets the JNIEnv for the current thread, attaching it permanently if needed.
pub fn get_env() -> Result<JNIEnv<'static>> {
    if !JVM_INITIALIZED.load(Ordering::SeqCst) {
        return Err(MinestomError::JvmInit("JVM not attached".into()));
    }

    let guard = JVM_INSTANCE.lock();
    if let Some(ref java_vm_arc) = *guard {
        let env = java_vm_arc
            .attach_current_thread_permanently()
            .map_err(|e| MinestomError::Jni(e))?;
        // SAFETY: The JavaVM is stored in a static and outlives all JNIEnv handles
        Ok(unsafe { std::mem::transmute(env) })
    } else {
        Err(MinestomError::JvmInit("No stored JVM instance".into()))
    }
}

/// A wrapper around a Java object that can be safely shared between threads.
#[derive(Clone)]
pub struct JavaObject {
    inner: Arc<GlobalRef>,
}

impl JavaObject {
    /// Creates a new JavaObject from a GlobalRef.
    pub fn new(global_ref: GlobalRef) -> Self {
        Self {
            inner: Arc::new(global_ref),
        }
    }

    /// Creates a new JavaObject from a JObject in the given environment.
    pub fn from_env<'local>(env: &mut JNIEnv<'local>, obj: JObject<'local>) -> Result<Self> {
        let local_ref = env.new_local_ref(&obj)?;
        let global_ref = env.new_global_ref(local_ref)?;
        Ok(Self::new(global_ref))
    }

    /// Gets a reference to the underlying JObject.
    /// This creates a NEW LOCAL REFERENCE that must be used within the current JNI scope.
    pub fn as_obj(&self) -> Result<JObject> {
        let env = get_env()?;
        let obj = env.new_local_ref(unsafe { JObject::from_raw(self.inner.as_raw()) })?;
        debug!("Created safe local reference for JavaObject");
        Ok(obj)
    }

    /// Calls a void method on this object with proper exception handling
    pub fn call_void_method<'arg_local>(
        &self,
        name: &str,
        sig: &str,
        args: &[JniValue<'arg_local>],
    ) -> Result<()> {
        let mut env = get_env()?;
        let _frame = env.push_local_frame(32)?;
        let target_obj_local = env.new_local_ref(&self.as_obj()?)?;

        // Step 1: Create and store all necessary owned local references for object/string arguments.
        let mut owned_java_args: Vec<JObject> = Vec::new();
        for arg in args.iter() {
            match arg {
                JniValue::Object(o) => owned_java_args.push(env.new_local_ref(o)?),
                JniValue::String(s) => owned_java_args.push(env.new_local_ref(s)?),
                _ => {}
            }
        }

        // Step 2: Create JValue args, borrowing from owned_java_args where necessary.
        let mut jvalue_args: Vec<jni::objects::JValue> = Vec::with_capacity(args.len());
        let mut owned_java_args_iter = owned_java_args.iter();

        for arg in args.iter() {
            match arg {
                JniValue::Object(_) => {
                    jvalue_args.push(jni::objects::JValue::from(
                        owned_java_args_iter.next().unwrap(),
                    ));
                }
                JniValue::String(_) => {
                    jvalue_args.push(jni::objects::JValue::from(
                        owned_java_args_iter.next().unwrap(),
                    ));
                }
                JniValue::Int(i) => jvalue_args.push(jni::objects::JValue::Int(*i)),
                JniValue::Long(l) => jvalue_args.push(jni::objects::JValue::Long(*l)),
                JniValue::Double(d) => jvalue_args.push(jni::objects::JValue::Double(*d)),
                JniValue::Float(f) => jvalue_args.push(jni::objects::JValue::Float(*f)),
                JniValue::Bool(b) => {
                    jvalue_args.push(jni::objects::JValue::Bool(if *b { 1 } else { 0 }))
                }
                JniValue::Void => jvalue_args.push(jni::objects::JValue::Void),
            }
        }

        env.call_method(target_obj_local, name, sig, &jvalue_args)?;
        check_exception(&mut env)?;
        Ok(())
    }

    /// Calls a method on this object that returns a Java object.
    pub fn call_object_method<'arg_local>(
        &self,
        name: &str,
        sig: &str,
        args: &[JniValue<'arg_local>],
    ) -> Result<JavaObject> {
        let mut env = get_env()?;
        let _frame = env.push_local_frame(32)?;
        let target_obj_local = env.new_local_ref(&self.as_obj()?)?;

        let mut owned_java_args: Vec<JObject> = Vec::new();
        for arg in args.iter() {
            match arg {
                JniValue::Object(o) => owned_java_args.push(env.new_local_ref(o)?),
                JniValue::String(s) => owned_java_args.push(env.new_local_ref(s)?),
                _ => {}
            }
        }

        let mut jvalue_args: Vec<jni::objects::JValue> = Vec::with_capacity(args.len());
        let mut owned_java_args_iter = owned_java_args.iter();

        for arg in args.iter() {
            match arg {
                JniValue::Object(_) => {
                    jvalue_args.push(jni::objects::JValue::from(
                        owned_java_args_iter.next().unwrap(),
                    ));
                }
                JniValue::String(_) => {
                    jvalue_args.push(jni::objects::JValue::from(
                        owned_java_args_iter.next().unwrap(),
                    ));
                }
                JniValue::Int(i) => jvalue_args.push(jni::objects::JValue::Int(*i)),
                JniValue::Long(l) => jvalue_args.push(jni::objects::JValue::Long(*l)),
                JniValue::Double(d) => jvalue_args.push(jni::objects::JValue::Double(*d)),
                JniValue::Float(f) => jvalue_args.push(jni::objects::JValue::Float(*f)),
                JniValue::Bool(b) => {
                    jvalue_args.push(jni::objects::JValue::Bool(if *b { 1 } else { 0 }))
                }
                JniValue::Void => jvalue_args.push(jni::objects::JValue::Void),
            }
        }

        let result = env.call_method(target_obj_local, name, sig, &jvalue_args)?;
        check_exception(&mut env)?;
        let result_obj_local = result.l()?;
        JavaObject::from_env(&mut env, result_obj_local)
    }

    /// Calls a method on this object that returns an integer.
    pub fn call_int_method<'arg_local>(
        &self,
        name: &str,
        sig: &str,
        args: &[JniValue<'arg_local>],
    ) -> Result<i32> {
        let mut env = get_env()?;
        let _frame = env.push_local_frame(32)?;
        let target_obj_local = env.new_local_ref(&self.as_obj()?)?;

        let mut owned_java_args: Vec<JObject> = Vec::new();
        for arg in args.iter() {
            match arg {
                JniValue::Object(o) => owned_java_args.push(env.new_local_ref(o)?),
                JniValue::String(s) => owned_java_args.push(env.new_local_ref(s)?),
                _ => {}
            }
        }

        let mut jvalue_args: Vec<jni::objects::JValue> = Vec::with_capacity(args.len());
        let mut owned_java_args_iter = owned_java_args.iter();

        for arg in args.iter() {
            match arg {
                JniValue::Object(_) => {
                    jvalue_args.push(jni::objects::JValue::from(
                        owned_java_args_iter.next().unwrap(),
                    ));
                }
                JniValue::String(_) => {
                    jvalue_args.push(jni::objects::JValue::from(
                        owned_java_args_iter.next().unwrap(),
                    ));
                }
                JniValue::Int(i) => jvalue_args.push(jni::objects::JValue::Int(*i)),
                JniValue::Long(l) => jvalue_args.push(jni::objects::JValue::Long(*l)),
                JniValue::Double(d) => jvalue_args.push(jni::objects::JValue::Double(*d)),
                JniValue::Float(f) => jvalue_args.push(jni::objects::JValue::Float(*f)),
                JniValue::Bool(b) => {
                    jvalue_args.push(jni::objects::JValue::Bool(if *b { 1 } else { 0 }))
                }
                JniValue::Void => jvalue_args.push(jni::objects::JValue::Void),
            }
        }

        let result = env.call_method(target_obj_local, name, sig, &jvalue_args)?;
        check_exception(&mut env)?;
        Ok(result.i()?)
    }

    /// Calls a method on this object that returns a boolean.
    pub fn call_bool_method<'arg_local>(
        &self,
        name: &str,
        sig: &str,
        args: &[JniValue<'arg_local>],
    ) -> Result<bool> {
        let mut env = get_env()?;
        let _frame = env.push_local_frame(32)?;
        let target_obj_local = env.new_local_ref(&self.as_obj()?)?;

        let mut owned_java_args: Vec<JObject> = Vec::new();
        for arg in args.iter() {
            match arg {
                JniValue::Object(o) => owned_java_args.push(env.new_local_ref(o)?),
                JniValue::String(s) => owned_java_args.push(env.new_local_ref(s)?),
                _ => {}
            }
        }

        let mut jvalue_args: Vec<jni::objects::JValue> = Vec::with_capacity(args.len());
        let mut owned_java_args_iter = owned_java_args.iter();

        for arg in args.iter() {
            match arg {
                JniValue::Object(_) => {
                    jvalue_args.push(jni::objects::JValue::from(
                        owned_java_args_iter.next().unwrap(),
                    ));
                }
                JniValue::String(_) => {
                    jvalue_args.push(jni::objects::JValue::from(
                        owned_java_args_iter.next().unwrap(),
                    ));
                }
                JniValue::Int(i) => jvalue_args.push(jni::objects::JValue::Int(*i)),
                JniValue::Long(l) => jvalue_args.push(jni::objects::JValue::Long(*l)),
                JniValue::Double(d) => jvalue_args.push(jni::objects::JValue::Double(*d)),
                JniValue::Float(f) => jvalue_args.push(jni::objects::JValue::Float(*f)),
                JniValue::Bool(b) => {
                    jvalue_args.push(jni::objects::JValue::Bool(if *b { 1 } else { 0 }))
                }
                JniValue::Void => jvalue_args.push(jni::objects::JValue::Void),
            }
        }

        let result = env.call_method(target_obj_local, name, sig, &jvalue_args)?;
        check_exception(&mut env)?;
        Ok(result.z()?)
    }
}

impl fmt::Debug for JavaObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JavaObject {{ ptr: {:?} }}", self.inner.as_raw())
    }
}

// A wrapper type that owns the JNI value and handles lifetime management
pub enum JniValue<'local> {
    Object(JObject<'local>),
    String(JString<'local>),
    Int(i32),
    Long(i64),
    Double(f64),
    Float(f32),
    Bool(bool),
    Void,
}

impl<'local> JniValue<'local> {
    pub fn from_jobject(obj: JObject<'local>) -> Self {
        JniValue::Object(obj)
    }

    pub fn from_jvalue(value: JValueGen<JObject<'local>>) -> Result<Self> {
        Ok(match value {
            JValueGen::Object(obj) => JniValue::Object(obj),
            JValueGen::Int(i) => JniValue::Int(i),
            JValueGen::Long(l) => JniValue::Long(l),
            JValueGen::Double(d) => JniValue::Double(d),
            JValueGen::Float(f) => JniValue::Float(f),
            JValueGen::Bool(b) => JniValue::Bool(b == 1),
            JValueGen::Void => JniValue::Void,
            _ => {
                return Err(MinestomError::JvmInit(
                    "Unsupported JValue type".to_string(),
                ))
            }
        })
    }

    pub fn as_jvalue(&'local self) -> JValueGen<&'local JObject<'local>> {
        match self {
            JniValue::Object(obj) => JValueGen::Object(obj),
            JniValue::String(s) => {
                JValueGen::Object(unsafe { std::mem::transmute::<&JString, &JObject>(s) })
            }
            JniValue::Int(i) => JValueGen::Int(*i),
            JniValue::Long(l) => JValueGen::Long(*l),
            JniValue::Double(d) => JValueGen::Double(*d),
            JniValue::Float(f) => JValueGen::Float(*f),
            JniValue::Bool(b) => JValueGen::Bool(if *b { 1 } else { 0 }),
            JniValue::Void => JValueGen::Void,
        }
    }

    pub fn l(&'local self) -> Result<&'local JObject<'local>> {
        match self {
            JniValue::Object(obj) => Ok(obj),
            JniValue::String(s) => Ok(unsafe { std::mem::transmute::<&JString, &JObject>(s) }),
            _ => Err(MinestomError::JvmInit("Not an object value".to_string())),
        }
    }

    pub fn i(&self) -> Result<i32> {
        match self {
            JniValue::Int(i) => Ok(*i),
            _ => Err(MinestomError::JvmInit("Not an integer value".to_string())),
        }
    }

    pub fn z(&self) -> Result<bool> {
        match self {
            JniValue::Bool(b) => Ok(*b),
            _ => Err(MinestomError::JvmInit("Not a boolean value".to_string())),
        }
    }

    pub fn d(&self) -> Result<f64> {
        match self {
            JniValue::Double(d) => Ok(*d),
            _ => Err(MinestomError::JvmInit("Not a double value".to_string())),
        }
    }

    pub fn f(&self) -> Result<f32> {
        match self {
            JniValue::Float(f) => Ok(*f),
            _ => Err(MinestomError::JvmInit("Not a float value".to_string())),
        }
    }
}

// Implement From traits for common types
impl<'local> From<i32> for JniValue<'local> {
    fn from(value: i32) -> Self {
        JniValue::Int(value)
    }
}

impl<'local> From<i64> for JniValue<'local> {
    fn from(value: i64) -> Self {
        JniValue::Long(value)
    }
}

impl<'local> From<f64> for JniValue<'local> {
    fn from(value: f64) -> Self {
        JniValue::Double(value)
    }
}

impl<'local> From<f32> for JniValue<'local> {
    fn from(value: f32) -> Self {
        JniValue::Float(value)
    }
}

impl<'local> From<bool> for JniValue<'local> {
    fn from(value: bool) -> Self {
        JniValue::Bool(value)
    }
}

impl<'local> From<JObject<'local>> for JniValue<'local> {
    fn from(obj: JObject<'local>) -> Self {
        JniValue::Object(obj)
    }
}

impl<'local> From<JString<'local>> for JniValue<'local> {
    fn from(s: JString<'local>) -> Self {
        JniValue::String(s)
    }
}

pub(crate) trait ToJava {
    fn to_java<'local>(&self, env: &mut JNIEnv<'local>) -> Result<JniValue<'local>>;
}

impl ToJava for str {
    fn to_java<'local>(&self, env: &mut JNIEnv<'local>) -> Result<JniValue<'local>> {
        Ok(JniValue::String(env.new_string(self)?))
    }
}

impl ToJava for String {
    fn to_java<'local>(&self, env: &mut JNIEnv<'local>) -> Result<JniValue<'local>> {
        self.as_str().to_java(env)
    }
}

impl ToJava for i32 {
    fn to_java<'local>(&self, _env: &mut JNIEnv<'local>) -> Result<JniValue<'local>> {
        Ok(JniValue::Int(*self))
    }
}

impl ToJava for i64 {
    fn to_java<'local>(&self, _env: &mut JNIEnv<'local>) -> Result<JniValue<'local>> {
        Ok(JniValue::Long(*self))
    }
}

impl ToJava for f64 {
    fn to_java<'local>(&self, _env: &mut JNIEnv<'local>) -> Result<JniValue<'local>> {
        Ok(JniValue::Double(*self))
    }
}

impl ToJava for bool {
    fn to_java<'local>(&self, _env: &mut JNIEnv<'local>) -> Result<JniValue<'local>> {
        Ok(JniValue::Bool(*self))
    }
}

impl<'a> ToJava for JObject<'a> {
    fn to_java<'local>(&self, env: &mut JNIEnv<'local>) -> Result<JniValue<'local>> {
        Ok(JniValue::Object(env.new_local_ref(self)?))
    }
}

impl ToJava for JavaObject {
    fn to_java<'local>(&self, env: &mut JNIEnv<'local>) -> Result<JniValue<'local>> {
        Ok(JniValue::Object(env.new_local_ref(&self.as_obj()?)?))
    }
}

/// Check if there's a Java exception and convert it to a Rust error
pub(crate) fn check_exception(env: &mut JNIEnv) -> Result<()> {
    if env.exception_check()? {
        let exception = env.exception_occurred()?;
        env.exception_clear()?;

        let msg = if let Ok(msg) =
            env.call_method(exception, "getMessage", "()Ljava/lang/String;", &[])
        {
            if let Ok(msg) = msg.l() {
                // Create a new local reference to ensure proper lifetime
                let msg_ref = env.new_local_ref(msg)?;
                let jstr = JString::from(msg_ref);
                // Store the result before the temporary is dropped
                let result = env
                    .get_string(&jstr)
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|_| "Unknown error".to_string());
                result
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
