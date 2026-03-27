//! Timing and RNG utilities for the rubato project.
//!
//! Contains TimerManager for game timing, JavaRandom (LCG port of java.util.Random),
//! and LR2Random (MT19937 port of LR2's RNG).

pub mod java_random;
pub mod lr2_random;
pub mod timer_manager;
