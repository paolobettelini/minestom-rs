use crate::Result;
use crate::jni_utils::{JavaObject, JniValue, get_env};

#[derive(Debug, Clone)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug)]
pub struct Pos {
    inner: JavaObject,
}

impl Position {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn to_pos(&self) -> Result<Pos> {
        let mut env = get_env()?;
        let pos_class = env.find_class("net/minestom/server/coordinate/Pos")?;
        let pos = env.new_object(
            pos_class,
            "(DDD)V",
            &[
                JniValue::Double(self.x).as_jvalue(),
                JniValue::Double(self.y).as_jvalue(),
                JniValue::Double(self.z).as_jvalue(),
            ],
        )?;
        Ok(Pos {
            inner: JavaObject::from_env(&mut env, pos)?,
        })
    }
}

impl Pos {
    pub fn new(inner: JavaObject) -> Self {
        Self { inner }
    }

    pub fn of(x: f64, y: f64, z: f64, yaw: f32, pitch: f32) -> Self {
        let mut env = get_env().unwrap();
        let pos_class = env.find_class("net/minestom/server/coordinate/Pos").unwrap();
        let pos = env.new_object(
            pos_class,
            "(DDDFF)V",
            &[
                JniValue::Double(x).as_jvalue(),
                JniValue::Double(y).as_jvalue(),
                JniValue::Double(z).as_jvalue(),
                JniValue::Float(yaw).as_jvalue(),
                JniValue::Float(pitch).as_jvalue(),
            ],
        ).unwrap();
        Pos {
            inner: JavaObject::from_env(&mut env, pos).unwrap(),
        }
    }

    pub fn to_position(&self) -> Result<Position> {
        let mut env = get_env()?;
        let obj = self.inner.as_obj()?;
        let x = env.call_method(&obj, "x", "()D", &[])?.d()?;
        let y = env.call_method(&obj, "y", "()D", &[])?.d()?;
        let z = env.call_method(&obj, "z", "()D", &[])?.d()?;
        Ok(Position::new(x, y, z))
    }

    pub fn inner(&self) -> &JavaObject {
        &self.inner
    }
}
