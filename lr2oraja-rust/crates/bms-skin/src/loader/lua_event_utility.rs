// Lua event observation utilities.
//
// Provides functions that create event observers for Lua skin scripts.
// These observe state transitions (boolean, timer on/off) and fire callbacks.
//
// Ported from Java EventUtility.java.

use std::cell::RefCell;
use std::rc::Rc;

use anyhow::Result;
use mlua::prelude::*;

/// Timer OFF sentinel value (matches Java Long.MIN_VALUE semantics).
pub const TIMER_OFF: i64 = i64::MIN;

/// Convert mlua::Error to anyhow::Error (LuaError lacks Send+Sync).
fn lua_err(e: mlua::Error) -> anyhow::Error {
    anyhow::anyhow!("{e}")
}

/// Register event utility functions into a Lua table.
///
/// Functions registered:
/// - `observe_turn_true(condition_fn, action_fn) -> event_fn`
/// - `observe_timer(timer_fn, action_fn) -> event_fn`
/// - `observe_timer_on(timer_fn, action_fn) -> event_fn`
/// - `observe_timer_off(timer_fn, action_fn) -> event_fn`
/// - `min_interval(interval_ms, action_fn) -> throttled_fn`
pub fn register_event_utilities(lua: &Lua, table: &LuaTable) -> Result<()> {
    // observe_turn_true: triggers when condition transitions false -> true
    table
        .set(
            "observe_turn_true",
            lua.create_function(
                |lua, (condition_fn, action_fn): (LuaFunction, LuaFunction)| {
                    let was_on = Rc::new(RefCell::new(false));
                    let was_on_clone = was_on.clone();
                    lua.create_function(move |_lua, ()| {
                        let is_on: bool = condition_fn.call(())?;
                        let mut prev = was_on_clone.borrow_mut();
                        if is_on && !*prev {
                            action_fn.call::<()>(())?;
                        }
                        *prev = is_on;
                        Ok(())
                    })
                },
            )
            .map_err(lua_err)?,
        )
        .map_err(lua_err)?;

    // observe_timer: triggers when timer value changes from TIMER_OFF
    table
        .set(
            "observe_timer",
            lua.create_function(|lua, (timer_fn, action_fn): (LuaFunction, LuaFunction)| {
                let prev_value = Rc::new(RefCell::new(TIMER_OFF));
                let prev_clone = prev_value.clone();
                lua.create_function(move |_lua, ()| {
                    let value: i64 = timer_fn.call(())?;
                    let mut prev = prev_clone.borrow_mut();
                    if value != *prev && value != TIMER_OFF {
                        action_fn.call::<()>(())?;
                    }
                    *prev = value;
                    Ok(())
                })
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)?;

    // observe_timer_on: triggers on OFF -> ON transition
    table
        .set(
            "observe_timer_on",
            lua.create_function(|lua, (timer_fn, action_fn): (LuaFunction, LuaFunction)| {
                let was_on = Rc::new(RefCell::new(false));
                let was_on_clone = was_on.clone();
                lua.create_function(move |_lua, ()| {
                    let value: i64 = timer_fn.call(())?;
                    let is_on = value != TIMER_OFF;
                    let mut prev = was_on_clone.borrow_mut();
                    if is_on && !*prev {
                        action_fn.call::<()>(())?;
                    }
                    *prev = is_on;
                    Ok(())
                })
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)?;

    // observe_timer_off: triggers on ON -> OFF transition
    table
        .set(
            "observe_timer_off",
            lua.create_function(|lua, (timer_fn, action_fn): (LuaFunction, LuaFunction)| {
                let was_off = Rc::new(RefCell::new(true));
                let was_off_clone = was_off.clone();
                lua.create_function(move |_lua, ()| {
                    let value: i64 = timer_fn.call(())?;
                    let is_off = value == TIMER_OFF;
                    let mut prev = was_off_clone.borrow_mut();
                    if is_off && !*prev {
                        action_fn.call::<()>(())?;
                    }
                    *prev = is_off;
                    Ok(())
                })
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)?;

    // min_interval: throttles action execution to minimum interval.
    // Takes interval in milliseconds, converts to microseconds internally.
    table
        .set(
            "min_interval",
            lua.create_function(|lua, (interval_ms, action_fn): (i64, LuaFunction)| {
                let interval_us = interval_ms * 1000;
                let last_exec = Rc::new(RefCell::new(TIMER_OFF));
                let last_clone = last_exec.clone();
                lua.create_function(move |_lua, now_us: i64| {
                    let mut last = last_clone.borrow_mut();
                    if *last == TIMER_OFF || (now_us - *last) >= interval_us {
                        action_fn.call::<()>(())?;
                        *last = now_us;
                    }
                    Ok(())
                })
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observe_turn_true_fires_on_false_to_true_transition() {
        let lua = Lua::new();
        let table = lua.create_table().unwrap();
        register_event_utilities(&lua, &table).unwrap();

        lua.globals().set("event_util", &table).unwrap();

        lua.load(
            r#"
            counter = 0
            state = false
            local observe = event_util.observe_turn_true(
                function() return state end,
                function() counter = counter + 1 end
            )
            -- false -> false: no fire
            observe()
            assert(counter == 0)
            -- false -> true: fire
            state = true
            observe()
            assert(counter == 1)
            -- true -> true: no fire
            observe()
            assert(counter == 1)
            -- true -> false -> true: fire again
            state = false
            observe()
            state = true
            observe()
            assert(counter == 2)
            "#,
        )
        .exec()
        .unwrap();
    }

    #[test]
    fn observe_timer_fires_on_value_change_from_off() {
        let lua = Lua::new();
        let table = lua.create_table().unwrap();
        register_event_utilities(&lua, &table).unwrap();

        lua.globals().set("event_util", &table).unwrap();
        lua.globals().set("TIMER_OFF", TIMER_OFF).unwrap();

        lua.load(
            r#"
            counter = 0
            timer_val = TIMER_OFF
            local observe = event_util.observe_timer(
                function() return timer_val end,
                function() counter = counter + 1 end
            )
            -- OFF -> OFF: no fire
            observe()
            assert(counter == 0)
            -- OFF -> 1000: fire
            timer_val = 1000
            observe()
            assert(counter == 1)
            -- 1000 -> 1000: no fire (same value)
            observe()
            assert(counter == 1)
            -- 1000 -> 2000: fire (changed)
            timer_val = 2000
            observe()
            assert(counter == 2)
            -- 2000 -> OFF: no fire (TIMER_OFF)
            timer_val = TIMER_OFF
            observe()
            assert(counter == 2)
            "#,
        )
        .exec()
        .unwrap();
    }

    #[test]
    fn observe_timer_on_fires_on_off_to_on() {
        let lua = Lua::new();
        let table = lua.create_table().unwrap();
        register_event_utilities(&lua, &table).unwrap();

        lua.globals().set("event_util", &table).unwrap();
        lua.globals().set("TIMER_OFF", TIMER_OFF).unwrap();

        lua.load(
            r#"
            counter = 0
            timer_val = TIMER_OFF
            local observe = event_util.observe_timer_on(
                function() return timer_val end,
                function() counter = counter + 1 end
            )
            observe()
            assert(counter == 0)
            timer_val = 5000
            observe()
            assert(counter == 1)
            -- Still ON, no fire
            observe()
            assert(counter == 1)
            "#,
        )
        .exec()
        .unwrap();
    }

    #[test]
    fn observe_timer_off_fires_on_on_to_off() {
        let lua = Lua::new();
        let table = lua.create_table().unwrap();
        register_event_utilities(&lua, &table).unwrap();

        lua.globals().set("event_util", &table).unwrap();
        lua.globals().set("TIMER_OFF", TIMER_OFF).unwrap();

        lua.load(
            r#"
            counter = 0
            timer_val = TIMER_OFF
            local observe = event_util.observe_timer_off(
                function() return timer_val end,
                function() counter = counter + 1 end
            )
            -- Already OFF at start: no fire
            observe()
            assert(counter == 0)
            -- Turn ON
            timer_val = 5000
            observe()
            assert(counter == 0)
            -- Turn OFF: fire
            timer_val = TIMER_OFF
            observe()
            assert(counter == 1)
            -- Still OFF: no fire
            observe()
            assert(counter == 1)
            "#,
        )
        .exec()
        .unwrap();
    }

    #[test]
    fn min_interval_throttles_execution() {
        let lua = Lua::new();
        let table = lua.create_table().unwrap();
        register_event_utilities(&lua, &table).unwrap();

        lua.globals().set("event_util", &table).unwrap();

        lua.load(
            r#"
            counter = 0
            local throttled = event_util.min_interval(100, function()
                counter = counter + 1
            end)
            -- First call always fires (interval_ms=100 -> 100000 us)
            throttled(0)
            assert(counter == 1)
            -- Too soon (50000 us = 50ms < 100ms)
            throttled(50000)
            assert(counter == 1)
            -- Enough time passed (100000 us = 100ms)
            throttled(100000)
            assert(counter == 2)
            -- Too soon again
            throttled(150000)
            assert(counter == 2)
            -- Enough again
            throttled(200000)
            assert(counter == 3)
            "#,
        )
        .exec()
        .unwrap();
    }
}
