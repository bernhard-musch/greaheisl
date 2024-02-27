//! From the hardware, we get an on/off state for each of the buttons,
//! encoded in "button flags" of generic type `F`.
//! Building your logic on these raw button states is possible but
//! cumbersome. The button processor helps you here. It recognizes
//! state changes and turns turns them into "events".
//!
//! ## Features
//!
//! - `std`: (default) uses standard library.
//!    *Note:* set `default-features = false` for no-std targets.

// no_std only when freature "std" is missing
#![cfg_attr(not(feature = "std"), no_std)]

/// access to the button signal
pub trait AccessButtonSignal {
    /// returns true if there is any button activity
    fn is_button_signal(&self) -> bool;
}

/// low level button state (each button can be pressed or not)
pub trait AccessButtonState {
    type ButtonFlags;
    /// returns a bitmask telling us which buttons are pressed
    fn get_button_flags(&self) -> Self::ButtonFlags;
}

mod button_processor;
pub use button_processor::{
    ButtonEvent, ButtonFlagsTrait, ButtonProcessor, ButtonProcessorOptions, ButtonState,
};

mod check_hold_button;
pub use check_hold_button::{CheckHoldButton, CheckHoldButtonResult};

use greaheisl_async::DurationMillis;
use greaheisl_async::{AccessTiming, Timer};

/// waits for a button press or repeat event, up to a given timeout
///
/// Returns nothing. Simply check the button processor state
/// to find out the reason why the function has terminated.
pub async fn wait_button_press_or_timeout<F: ButtonFlagsTrait>(
    sys: &impl AccessTiming,
    btns: &ButtonProcessor<F>,
    timeout: DurationMillis,
) {
    let timeout_timer = Timer::new(sys, timeout);
    while timeout_timer.yield_if_time_left().await {
        //println!("   {}  {:?}   ",sys.get_instant().into_inner(),btns.event()); // for debugging
        match btns.event() {
            ButtonEvent::Press(_) | ButtonEvent::Repeat(_) => break,
            _ => {}
        }
    }
}
