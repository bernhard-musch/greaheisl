// no_std only when freature "std" is missing
#![cfg_attr(not(feature = "std"), no_std)]

pub trait AccessButtonSignal {
    fn is_button_signal(&self) -> bool;
}

/// low level button state (each button can be pressed or not)
pub trait AccessButtonState {
    type ButtonFlags;
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
