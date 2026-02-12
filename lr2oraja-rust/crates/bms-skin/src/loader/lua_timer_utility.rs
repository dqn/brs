// Lua timer utility functions.
//
// Provides helper functions for timer management in Lua skin scripts.
// Includes elapsed time calculation, timer state checks, and passive timers.
//
// Ported from Java TimerUtility.java.

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

/// Register timer utility functions into a Lua table.
///
/// Functions:
/// - `now_timer(timer_value) -> elapsed_us` (0 if OFF)
/// - `is_timer_on(timer_value) -> bool`
/// - `is_timer_off(timer_value) -> bool`
/// - `timer_observe_boolean(condition_fn) -> timer_fn` (passive timer from boolean)
/// - `new_passive_timer() -> {timer, turn_on, turn_on_reset, turn_off}`
pub fn register_timer_utilities(
    lua: &Lua,
    table: &LuaTable,
    get_now_us: LuaFunction,
) -> Result<()> {
    // now_timer: returns elapsed microseconds since timer started
    let get_now_clone = get_now_us.clone();
    table
        .set(
            "now_timer",
            lua.create_function(move |_lua, timer_value: i64| {
                if timer_value != TIMER_OFF {
                    let now: i64 = get_now_clone.call(())?;
                    Ok(now - timer_value)
                } else {
                    Ok(0i64)
                }
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)?;

    // is_timer_on: checks if timer is active
    table
        .set(
            "is_timer_on",
            lua.create_function(|_lua, timer_value: i64| Ok(timer_value != TIMER_OFF))
                .map_err(lua_err)?,
        )
        .map_err(lua_err)?;

    // is_timer_off: checks if timer is inactive
    table
        .set(
            "is_timer_off",
            lua.create_function(|_lua, timer_value: i64| Ok(timer_value == TIMER_OFF))
                .map_err(lua_err)?,
        )
        .map_err(lua_err)?;

    // timer_observe_boolean: creates passive timer from boolean condition
    let get_now_clone2 = get_now_us.clone();
    table
        .set(
            "timer_observe_boolean",
            lua.create_function(move |lua, condition_fn: LuaFunction| {
                let timer_value = Rc::new(RefCell::new(TIMER_OFF));
                let tv_clone = timer_value.clone();
                let now_fn = get_now_clone2.clone();
                lua.create_function(move |_lua, ()| {
                    let is_on: bool = condition_fn.call(())?;
                    let mut tv = tv_clone.borrow_mut();
                    if is_on && *tv == TIMER_OFF {
                        let now: i64 = now_fn.call(())?;
                        *tv = now;
                    } else if !is_on && *tv != TIMER_OFF {
                        *tv = TIMER_OFF;
                    }
                    Ok(*tv)
                })
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)?;

    // new_passive_timer: creates a controllable passive timer object
    let get_now_clone3 = get_now_us.clone();
    table
        .set(
            "new_passive_timer",
            lua.create_function(move |lua, ()| {
                let timer_value = Rc::new(RefCell::new(TIMER_OFF));
                let result = lua.create_table()?;

                // timer() -> current value (microseconds or TIMER_OFF)
                let tv1 = timer_value.clone();
                result.set(
                    "timer",
                    lua.create_function(move |_lua, ()| Ok(*tv1.borrow()))?,
                )?;

                // turn_on() -> activates if not already ON
                let tv2 = timer_value.clone();
                let now_fn2 = get_now_clone3.clone();
                result.set(
                    "turn_on",
                    lua.create_function(move |_lua, ()| {
                        let mut tv = tv2.borrow_mut();
                        if *tv == TIMER_OFF {
                            *tv = now_fn2.call(())?;
                        }
                        Ok(())
                    })?,
                )?;

                // turn_on_reset() -> activates and resets elapsed time
                let tv3 = timer_value.clone();
                let now_fn3 = get_now_clone3.clone();
                result.set(
                    "turn_on_reset",
                    lua.create_function(move |_lua, ()| {
                        let mut tv = tv3.borrow_mut();
                        *tv = now_fn3.call(())?;
                        Ok(())
                    })?,
                )?;

                // turn_off() -> deactivates timer
                let tv4 = timer_value.clone();
                result.set(
                    "turn_off",
                    lua.create_function(move |_lua, ()| {
                        *tv4.borrow_mut() = TIMER_OFF;
                        Ok(())
                    })?,
                )?;

                Ok(result)
            })
            .map_err(lua_err)?,
        )
        .map_err(lua_err)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_lua_with_clock(initial_us: i64) -> (Lua, LuaTable) {
        let lua = Lua::new();
        lua.globals().set("_clock_us", initial_us).unwrap();
        let get_now = lua
            .create_function(|lua, ()| {
                let now: i64 = lua.globals().get("_clock_us")?;
                Ok(now)
            })
            .unwrap();
        let table = lua.create_table().unwrap();
        register_timer_utilities(&lua, &table, get_now).unwrap();
        lua.globals().set("timer_util", &table).unwrap();
        lua.globals().set("TIMER_OFF", TIMER_OFF).unwrap();
        (lua, table)
    }

    #[test]
    fn now_timer_returns_elapsed() {
        let (lua, _) = setup_lua_with_clock(10_000_000);

        lua.load(
            r#"
            local elapsed = timer_util.now_timer(5000000)
            assert(elapsed == 5000000, "expected 5000000, got " .. elapsed)
            "#,
        )
        .exec()
        .unwrap();
    }

    #[test]
    fn now_timer_returns_zero_when_off() {
        let (lua, _) = setup_lua_with_clock(10_000_000);

        lua.load(
            r#"
            local elapsed = timer_util.now_timer(TIMER_OFF)
            assert(elapsed == 0, "expected 0, got " .. elapsed)
            "#,
        )
        .exec()
        .unwrap();
    }

    #[test]
    fn is_timer_on_and_off() {
        let (lua, _) = setup_lua_with_clock(0);

        lua.load(
            r#"
            assert(timer_util.is_timer_on(1000) == true)
            assert(timer_util.is_timer_on(TIMER_OFF) == false)
            assert(timer_util.is_timer_off(TIMER_OFF) == true)
            assert(timer_util.is_timer_off(1000) == false)
            "#,
        )
        .exec()
        .unwrap();
    }

    #[test]
    fn timer_observe_boolean_lifecycle() {
        let (lua, _) = setup_lua_with_clock(1_000_000);

        lua.load(
            r#"
            local state = false
            local timer_fn = timer_util.timer_observe_boolean(function() return state end)

            -- Initially OFF
            local v = timer_fn()
            assert(v == TIMER_OFF)

            -- Turn on: captures current time
            state = true
            v = timer_fn()
            assert(v == 1000000, "expected 1000000, got " .. v)

            -- Stays at same start time
            _clock_us = 2000000
            v = timer_fn()
            assert(v == 1000000)

            -- Turn off
            state = false
            v = timer_fn()
            assert(v == TIMER_OFF)
            "#,
        )
        .exec()
        .unwrap();
    }

    #[test]
    fn passive_timer_lifecycle() {
        let (lua, _) = setup_lua_with_clock(1_000_000);

        lua.load(
            r#"
            local pt = timer_util.new_passive_timer()

            -- Initially OFF
            assert(pt.timer() == TIMER_OFF)

            -- Turn on
            pt.turn_on()
            assert(pt.timer() == 1000000)

            -- turn_on again is no-op
            _clock_us = 2000000
            pt.turn_on()
            assert(pt.timer() == 1000000, "should not change")

            -- turn_on_reset resets to current time
            pt.turn_on_reset()
            assert(pt.timer() == 2000000)

            -- Turn off
            pt.turn_off()
            assert(pt.timer() == TIMER_OFF)
            "#,
        )
        .exec()
        .unwrap();
    }
}
