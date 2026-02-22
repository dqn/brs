use mlua::prelude::*;

use crate::property::boolean_property_factory;
use crate::property::float_property_factory;
use crate::property::integer_property_factory;
use crate::property::string_property_factory;
use crate::stubs::MainState;

/// Main state accessor for Lua
///
/// Translated from MainStateAccessor.java (319 lines)
/// Provides Lua functions to access game state values from MainState.
/// Exports functions: option, number, float_number, text, offset, timer,
/// timer_off_value, time, event_index
///
/// NOTE: rate, exscore, volume, judge, gauge, audio functions require
/// ScoreDataProperty / AudioConfig / BMSPlayer access on MainState trait,
/// which are not yet available. These will be wired when MainState trait is extended.
///
/// Timer off value constant (Long.MIN_VALUE in Java)
pub const TIMER_OFF_VALUE: i64 = i64::MIN;

/// Wrapper for raw MainState pointer to implement Send/Sync.
/// SAFETY: The MainState is accessed single-threaded in beatoraja's skin system,
/// and the MainState reference outlives the Lua VM.
#[derive(Clone, Copy)]
struct StatePtr(*const dyn MainState);
unsafe impl Send for StatePtr {}
unsafe impl Sync for StatePtr {}

pub struct MainStateAccessor {
    state_ptr: StatePtr,
}

impl MainStateAccessor {
    pub fn new(state: &dyn MainState) -> Self {
        // SAFETY: We erase the lifetime of the trait object pointer.
        // The caller guarantees that state outlives MainStateAccessor.
        let ptr: *const dyn MainState = state;
        let ptr: *const dyn MainState = unsafe { std::mem::transmute(ptr) };
        Self {
            state_ptr: StatePtr(ptr),
        }
    }

    /// Export all accessor functions to a Lua table
    pub fn export(&self, lua: &Lua, table: &LuaTable) {
        let result: Result<(), LuaError> = (|| {
            let sp = self.state_ptr;

            // option(id) -> boolean
            let option_func = lua.create_function(move |_, id: i32| {
                let state = unsafe { &*sp.0 };
                Ok(option_fn(state, id))
            })?;
            table.set("option", option_func)?;

            // number(id) -> integer
            let sp = self.state_ptr;
            let number_func = lua.create_function(move |_, id: i32| {
                let state = unsafe { &*sp.0 };
                Ok(number_fn(state, id))
            })?;
            table.set("number", number_func)?;

            // float_number(id) -> float
            let sp = self.state_ptr;
            let float_number_func = lua.create_function(move |_, id: f64| {
                let state = unsafe { &*sp.0 };
                Ok(float_number_fn(state, id as i32))
            })?;
            table.set("float_number", float_number_func)?;

            // text(id) -> string
            let sp = self.state_ptr;
            let text_func = lua.create_function(move |_, id: i32| {
                let state = unsafe { &*sp.0 };
                Ok(text_fn(state, id))
            })?;
            table.set("text", text_func)?;

            // offset(id) -> table {x, y, w, h, r, a}
            let sp = self.state_ptr;
            let offset_func = lua.create_function(move |lua, id: i32| {
                let state = unsafe { &*sp.0 };
                let tbl = lua.create_table()?;
                if let Some(offset) = state.get_offset_value(id) {
                    tbl.set("x", offset.x as f64)?;
                    tbl.set("y", offset.y as f64)?;
                    tbl.set("w", offset.w as f64)?;
                    tbl.set("h", offset.h as f64)?;
                    tbl.set("r", offset.r as f64)?;
                    tbl.set("a", offset.a as f64)?;
                } else {
                    tbl.set("x", 0.0)?;
                    tbl.set("y", 0.0)?;
                    tbl.set("w", 0.0)?;
                    tbl.set("h", 0.0)?;
                    tbl.set("r", 0.0)?;
                    tbl.set("a", 0.0)?;
                }
                Ok(tbl)
            })?;
            table.set("offset", offset_func)?;

            // timer(id) -> integer (micro sec)
            let sp = self.state_ptr;
            let timer_func = lua.create_function(move |_, id: i32| {
                let state = unsafe { &*sp.0 };
                Ok(state.get_timer().get_micro_timer(id))
            })?;
            table.set("timer", timer_func)?;

            // timer_off_value constant
            table.set("timer_off_value", TIMER_OFF_VALUE)?;

            // time() -> integer (current micro time)
            let sp = self.state_ptr;
            let time_func = lua.create_function(move |_, ()| {
                let state = unsafe { &*sp.0 };
                Ok(state.get_timer().get_now_micro_time())
            })?;
            table.set("time", time_func)?;

            // event_index(id) -> integer
            let sp = self.state_ptr;
            let event_index_func = lua.create_function(move |_, id: i32| {
                let state = unsafe { &*sp.0 };
                Ok(event_index_fn(state, id))
            })?;
            table.set("event_index", event_index_func)?;

            // NOTE: rate, exscore, rate_best, exscore_best, rate_rival, exscore_rival,
            // volume_sys/key/bg, set_volume_sys/key/bg, judge, gauge, gauge_type,
            // audio_play/loop/stop, set_timer, event_exec
            // require ScoreDataProperty/AudioConfig/BMSPlayer access on MainState trait.
            // These will be wired when the MainState trait is extended in Phase 18.

            Ok(())
        })();
        if let Err(e) = result {
            log::warn!("MainStateAccessor::export failed: {}", e);
        }
    }
}

/// option function - Gets OPTION_* boolean by ID
pub fn option_fn(state: &dyn MainState, id: i32) -> bool {
    if let Some(prop) = boolean_property_factory::get_boolean_property(id) {
        prop.get(state)
    } else {
        false
    }
}

/// number function - Gets NUMBER_* integer by ID
pub fn number_fn(state: &dyn MainState, id: i32) -> i32 {
    if let Some(prop) = integer_property_factory::get_integer_property_by_id(id) {
        prop.get(state)
    } else {
        0
    }
}

/// float_number function - Gets SLIDER_*/BARGRAPH_* float by ID
pub fn float_number_fn(state: &dyn MainState, id: i32) -> f32 {
    if let Some(prop) = float_property_factory::get_rate_property_by_id(id) {
        prop.get(state)
    } else {
        0.0
    }
}

/// text function - Gets STRING_* text by ID
pub fn text_fn(state: &dyn MainState, id: i32) -> String {
    if let Some(prop) = string_property_factory::get_string_property_by_id(id) {
        prop.get(state)
    } else {
        String::new()
    }
}

/// event_index function - Gets event/button index by ID
pub fn event_index_fn(state: &dyn MainState, id: i32) -> i32 {
    if let Some(prop) = integer_property_factory::get_image_index_property_by_id(id) {
        prop.get(state)
    } else {
        0
    }
}
