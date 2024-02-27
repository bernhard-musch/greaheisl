//! # `greaheisl` library
//!
//! This is the major rust part of the `greaheisl` project.
//! This crate can be used in two ways:
//! * It can be compiled as a no-std library and statically linked into the C++ Arduino sketch.
//!   The corresponding C API is defined in module [interface_c].
//! * It can be compiled as a normal library to be used with the `greaheisl-emu` test application.
//!
//! ## Features
//!
//! - `std`: (default) uses standard library.
//!    *Note:* set `default-features = false` for no-std targets.

// no_std only when freature "std" is missing
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

// #[macro_use]
// pub mod greaheisl_async;

pub mod interface_c;

pub mod system;

// only needed when we build for embedded device
#[cfg(not(feature = "std"))]
mod delegating_alloc;

pub mod ui;

// panic handling is only needed when we build for embedded device
#[cfg(not(feature = "std"))]
mod panic_handling;

// tests with standard library
#[cfg(feature = "std")]
#[cfg(test)]
mod tests;

use crate::system::{AccessOutputStates, AccessRtc, NUM_RELAYS};
use core::cell::RefCell;
use greaheisl_async::{join2, AccessTiming, Timer};
use greaheisl_async::{DurationMillis, InstantMillis};
use system::System;
use ui::run_ui;
// use serde::{Serialize, Deserialize};

const MAX_SCHEDULED_ENTRIES: usize = 3;

/// main structure containing the current settings
#[derive(Clone, Default)] // later also ,Serialize,Deserialize)]
pub struct Settings {
    /// timers starting immediately
    pub immediate_out: [Option<ImmediateOutEntry>; NUM_RELAYS],
    /// timers starting daily on scheduled time
    pub scheduled_out: [[ScheduledOutEntry; MAX_SCHEDULED_ENTRIES]; NUM_RELAYS],
}

/// setting for timers starting immediately
#[derive(Debug, Clone)] //later also ,Serialize,Deserialize)]
pub struct ImmediateOutEntry {
    /// a time stamp when the timer was set
    pub start: InstantMillis,
    /// the selected duration
    pub duration: DurationMillis,
}

/// setting for scheduled timers
#[derive(Debug, Clone, Default)] // later also ,Serialize,Deserialize)]
pub struct ScheduledOutEntry {
    /// hour of the specified daily the start time
    pub start_hour: u8,
    /// minute of the specified daily the start time
    pub start_minute: u8,
    /// specified duration
    pub duration: DurationMillis,
}

/// main "task"
///
/// This is the entry point. From here all other tasks are fork off.
/// * the tasks driving the UI
/// * the task driving the relays.
pub async fn run(sys: impl System) {
    let settings = RefCell::new(Settings::default());
    join2(run_ui(&sys, &settings), watch_output(&sys, &settings)).await;
}

/// determines the frequency how often we check if relays need to change state
const OUTPUT_UPDATE_DELAY: DurationMillis = 2000;

/// regularly check whether the relays states need to change
async fn watch_output(
    sys: &(impl AccessOutputStates + AccessTiming + AccessRtc),
    settings: &RefCell<Settings>,
) {
    let mut old_relays_state = [false; NUM_RELAYS];
    let mut scheduled_out_stop =
        [[Option::<InstantMillis>::None; MAX_SCHEDULED_ENTRIES]; NUM_RELAYS];
    loop {
        let mut new_relays_state = [false; NUM_RELAYS];
        // check immediate entries
        {
            let mut settings = settings.borrow_mut();
            for (i, entry) in settings.immediate_out.iter_mut().enumerate() {
                let mut entry_copy = entry.clone();
                normalize_immediate_out_entry(&mut entry_copy, sys.get_instant());
                new_relays_state[i] = entry_copy.is_some();
                if entry_copy.is_none() {
                    // this timer is no longer active and the entry can be deleted
                    *entry = None;
                }
            }
        }
        // check scheduled entries
        {
            let rtc_time = sys.get_rtc();
            let scheduled_entries = &settings.borrow().scheduled_out;
            for i_relays in 0..NUM_RELAYS {
                for i_scheduled in 0..MAX_SCHEDULED_ENTRIES {
                    if let Some(stop_instant) = scheduled_out_stop[i_relays][i_scheduled] {
                        let time_left = stop_instant - sys.get_instant();
                        if time_left >= 0 {
                            new_relays_state[i_relays] = true;
                        } else {
                            scheduled_out_stop[i_relays][i_scheduled] = None;
                        }
                    } else {
                        let entry = &scheduled_entries[i_relays][i_scheduled];
                        if entry.duration > 0
                            && entry.start_hour == rtc_time.hour
                            && entry.start_minute == rtc_time.minute
                        {
                            scheduled_out_stop[i_relays][i_scheduled] =
                                Some(sys.get_instant() + entry.duration);
                            new_relays_state[i_relays] = true;
                        }
                    }
                }
            }
        }
        if old_relays_state != new_relays_state {
            sys.set_relay_states(&new_relays_state);
            old_relays_state = new_relays_state;
        }
        Timer::new(sys, OUTPUT_UPDATE_DELAY).wait().await;
    }
}

fn normalize_immediate_out_entry(entry: &mut Option<ImmediateOutEntry>, now: InstantMillis) {
    let Some(ImmediateOutEntry { start, duration }) = entry else {
        return;
    };
    let time_left = *duration - (now - *start);
    if time_left <= 0 {
        *entry = None;
    } else {
        *entry = Some(ImmediateOutEntry {
            start: now,
            duration: time_left,
        });
    }
}
