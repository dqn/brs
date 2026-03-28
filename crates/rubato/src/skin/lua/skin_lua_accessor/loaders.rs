use std::sync::{Arc, Mutex};

use mlua::prelude::*;

use crate::skin::property::boolean_property::BooleanProperty;
use crate::skin::property::event::Event;
use crate::skin::property::float_property::FloatPropertyEnum;
use crate::skin::property::float_writer::FloatWriter;
use crate::skin::property::integer_property::IntegerProperty;
use crate::skin::property::string_property::StringProperty;
use crate::skin::property::timer_property::TimerPropertyEnum;

use super::SkinLuaAccessor;
use super::property_impls::{
    LuaBooleanProperty, LuaEvent, LuaFloatProperty, LuaFloatWriter, LuaIntegerProperty,
    LuaStringProperty, LuaTimerProperty,
};

impl SkinLuaAccessor {
    /// Load a BooleanProperty from a Lua script string
    pub fn load_boolean_property_from_script(
        &self,
        script: &str,
    ) -> Option<Box<dyn BooleanProperty>> {
        let full_script = format!("return {}", script);
        match self.lua.load(&full_script).into_function() {
            Ok(func) => self.load_boolean_property_from_lua_function(func),
            Err(e) => {
                log::warn!("Lua parse error (boolean property): {}", e);
                None
            }
        }
    }

    /// Load a BooleanProperty from a Lua function
    pub fn load_boolean_property_from_function(
        &self,
        func: LuaFunction,
    ) -> Option<Box<dyn BooleanProperty>> {
        self.load_boolean_property_from_lua_function(func)
    }

    fn load_boolean_property_from_lua_function(
        &self,
        func: LuaFunction,
    ) -> Option<Box<dyn BooleanProperty>> {
        let func_key = self.lua.create_registry_value(func).ok()?;
        Some(Box::new(LuaBooleanProperty {
            func_key: Arc::new(Mutex::new(func_key)),
            lua: Arc::clone(&self.lua),
            creation_thread_id: std::thread::current().id(),
        }))
    }

    /// Load an IntegerProperty from a Lua script string
    pub fn load_integer_property_from_script(
        &self,
        script: &str,
    ) -> Option<Box<dyn IntegerProperty>> {
        let full_script = format!("return {}", script);
        match self.lua.load(&full_script).into_function() {
            Ok(func) => self.load_integer_property_from_lua_function(func),
            Err(e) => {
                log::warn!("Lua parse error (integer property): {}", e);
                None
            }
        }
    }

    /// Load an IntegerProperty from a Lua function
    pub fn load_integer_property_from_function(
        &self,
        func: LuaFunction,
    ) -> Option<Box<dyn IntegerProperty>> {
        self.load_integer_property_from_lua_function(func)
    }

    fn load_integer_property_from_lua_function(
        &self,
        func: LuaFunction,
    ) -> Option<Box<dyn IntegerProperty>> {
        let func_key = self.lua.create_registry_value(func).ok()?;
        Some(Box::new(LuaIntegerProperty {
            func_key: Arc::new(Mutex::new(func_key)),
            lua: Arc::clone(&self.lua),
            creation_thread_id: std::thread::current().id(),
        }))
    }

    /// Load a FloatProperty from a Lua script string
    pub fn load_float_property_from_script(&self, script: &str) -> Option<FloatPropertyEnum> {
        let full_script = format!("return {}", script);
        match self.lua.load(&full_script).into_function() {
            Ok(func) => self.load_float_property_from_lua_function(func),
            Err(e) => {
                log::warn!("Lua parse error (float property): {}", e);
                None
            }
        }
    }

    /// Load a FloatProperty from a Lua function
    pub fn load_float_property_from_function(
        &self,
        func: LuaFunction,
    ) -> Option<FloatPropertyEnum> {
        self.load_float_property_from_lua_function(func)
    }

    fn load_float_property_from_lua_function(
        &self,
        func: LuaFunction,
    ) -> Option<FloatPropertyEnum> {
        let func_key = self.lua.create_registry_value(func).ok()?;
        Some(FloatPropertyEnum::Lua(LuaFloatProperty {
            func_key: Arc::new(Mutex::new(func_key)),
            lua: Arc::clone(&self.lua),
            creation_thread_id: std::thread::current().id(),
        }))
    }

    /// Load a StringProperty from a Lua script string
    pub fn load_string_property_from_script(
        &self,
        script: &str,
    ) -> Option<Box<dyn StringProperty>> {
        let full_script = format!("return {}", script);
        match self.lua.load(&full_script).into_function() {
            Ok(func) => self.load_string_property_from_lua_function(func),
            Err(e) => {
                log::warn!("Lua parse error (string property): {}", e);
                None
            }
        }
    }

    /// Load a StringProperty from a Lua function
    pub fn load_string_property_from_function(
        &self,
        func: LuaFunction,
    ) -> Option<Box<dyn StringProperty>> {
        self.load_string_property_from_lua_function(func)
    }

    fn load_string_property_from_lua_function(
        &self,
        func: LuaFunction,
    ) -> Option<Box<dyn StringProperty>> {
        let func_key = self.lua.create_registry_value(func).ok()?;
        Some(Box::new(LuaStringProperty {
            func_key: Arc::new(Mutex::new(func_key)),
            lua: Arc::clone(&self.lua),
            creation_thread_id: std::thread::current().id(),
        }))
    }

    /// Load a TimerProperty from a Lua script string
    /// If the script returns a function, that function is used as a timer function.
    /// Otherwise, the script itself is regarded as a timer function.
    /// A timer function returns start time in microseconds if on, or i64::MIN if off.
    pub fn load_timer_property_from_script(&self, script: &str) -> Option<TimerPropertyEnum> {
        let full_script = format!("return {}", script);
        match self.lua.load(&full_script).into_function() {
            Ok(func) => {
                // Trial call: if the result is a function, use that instead
                match func.call::<LuaValue>(()) {
                    Ok(LuaValue::Function(inner_func)) => {
                        self.load_timer_property_from_lua_function(inner_func)
                    }
                    Ok(_) => {
                        // The script itself returns a number, use the script as timer function
                        self.load_timer_property_from_lua_function(func)
                    }
                    Err(e) => {
                        log::warn!("Lua parse error (timer property trial call): {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                log::warn!("Lua parse error (timer property): {}", e);
                None
            }
        }
    }

    /// Load a TimerProperty from a Lua function
    pub fn load_timer_property_from_function(
        &self,
        func: LuaFunction,
    ) -> Option<TimerPropertyEnum> {
        self.load_timer_property_from_lua_function(func)
    }

    fn load_timer_property_from_lua_function(
        &self,
        func: LuaFunction,
    ) -> Option<TimerPropertyEnum> {
        let func_key = self.lua.create_registry_value(func).ok()?;
        Some(TimerPropertyEnum::Lua(LuaTimerProperty {
            func_key: Arc::new(Mutex::new(func_key)),
            lua: Arc::clone(&self.lua),
            creation_thread_id: std::thread::current().id(),
        }))
    }

    /// Load an Event from a Lua script string
    pub fn load_event_from_script(&self, script: &str) -> Option<Box<dyn Event>> {
        match self.lua.load(script).into_function() {
            Ok(func) => self.load_event_from_lua_function(func),
            Err(e) => {
                log::warn!("Lua parse error (event): {}", e);
                None
            }
        }
    }

    /// Load an Event from a Lua function
    pub fn load_event_from_function(&self, func: LuaFunction) -> Option<Box<dyn Event>> {
        self.load_event_from_lua_function(func)
    }

    fn load_event_from_lua_function(&self, func: LuaFunction) -> Option<Box<dyn Event>> {
        let func_key = self.lua.create_registry_value(func).ok()?;
        Some(Box::new(LuaEvent {
            func_key: Arc::new(Mutex::new(func_key)),
            lua: Arc::clone(&self.lua),
            creation_thread_id: std::thread::current().id(),
        }))
    }

    /// Load a FloatWriter from a Lua script string
    pub fn load_float_writer_from_script(&self, script: &str) -> Option<Box<dyn FloatWriter>> {
        match self.lua.load(script).into_function() {
            Ok(func) => self.load_float_writer_from_lua_function(func),
            Err(e) => {
                log::warn!("Lua parse error (float writer): {}", e);
                None
            }
        }
    }

    /// Load a FloatWriter from a Lua function
    pub fn load_float_writer_from_function(
        &self,
        func: LuaFunction,
    ) -> Option<Box<dyn FloatWriter>> {
        self.load_float_writer_from_lua_function(func)
    }

    fn load_float_writer_from_lua_function(
        &self,
        func: LuaFunction,
    ) -> Option<Box<dyn FloatWriter>> {
        let func_key = self.lua.create_registry_value(func).ok()?;
        Some(Box::new(LuaFloatWriter {
            func_key: Arc::new(Mutex::new(func_key)),
            lua: Arc::clone(&self.lua),
            creation_thread_id: std::thread::current().id(),
        }))
    }
}
