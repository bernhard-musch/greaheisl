//! response to the push button keys

use bitmask_enum::bitmask;
use greaheisl_async::DurationMillis;
use greaheisl_async::{AccessTiming, Timer};
use greaheisl_button_processor::{ButtonEvent, ButtonFlagsTrait, ButtonProcessor};

/// Sometimes additional functionality becomes available if the
/// button is held down for a the number of milliseconds defined here.
pub const BUTTON_HOLD_DURATION: DurationMillis = 2000;

/// the buttons and their meaning
#[bitmask(u8)]
pub enum ButtonFlags {
    /// = back / exit / cancel
    Escape,
    /// previous item in the list of choices
    Prev,
    /// next item in the list of choices
    Next,
    /// = open menu / change value / confirm
    Enter,
}

/// the number of buttons
pub const NUM_BUTTONS: u32 = 4;
// We need to write down the number `NUM_BUTTONS` by hand,
// so that `cbindgen` can pick it up correctly.
// However, we can at least check consistency with the definition of `ButtonFlags`.
static_assertions::const_assert_eq!(NUM_BUTTONS, ButtonFlags::full().bits().count_ones());

impl ButtonFlagsTrait for ButtonFlags {
    fn is_none(&self) -> bool {
        self.is_none()
    }

    fn contains(&self, other: Self) -> bool {
        self.contains(other)
    }
}

/*
impl<T: AccessExecutorSignals<SignalFlags>> AccessButtonSignal for T  {
    fn is_button_signal(&self) -> bool {
        self.get_executor_signals().contains(SignalFlags::Button)
    }
}
*/

pub type SysButtonProcessor = ButtonProcessor<ButtonFlags>;

/// modified function to wait for a button press or repeat event, up to a given timeout
///
/// Unlike [`wait_button_press_or_timeout`], only the [`ButtonEvent::Repeat`] events
/// of the [`ButtonFlags::Prev`] and [`ButtonFlags::Next`] buttons are recognized,
/// other [`ButtonEvent::Repeat`] are ignored.
///
/// Returns nothing. Simply check the button processor state
/// to find out the reason why the function has terminated.
pub async fn wait_button_press2_or_timeout(
    sys: &impl AccessTiming,
    btns: &SysButtonProcessor,
    timeout: DurationMillis,
) {
    let timeout_timer = Timer::new(sys, timeout);
    while timeout_timer.yield_if_time_left().await {
        //println!("   {}  {:?}   ",sys.get_instant().into_inner(),btns.event()); // for debugging
        match btns.event() {
            ButtonEvent::Press(_) => break,
            ButtonEvent::Repeat(ButtonFlags::Next) => break,
            ButtonEvent::Repeat(ButtonFlags::Prev) => break,
            _ => {}
        }
    }
}

/*
pub async fn wait_button(
    sys: &impl AccessTiming+AccessSignalFlags,
    ber: &mut ButtonEventReader<'_>
) -> Option<(ButtonEvent,ButtonState)> {
    loop {
        let ev = ber.read_event();
        match ev {
            ButtonEvent::None => {},
            ev_not_none => {
                return Some((ev_not_none,ber.state()));
            }
        }
        super::wait_event(sys, SignalFlags::Button);
    }
}

pub async fn wait_button_timeout(
    sys: &impl AccessTiming,
    ber: &mut ButtonEventReader<'_>,
    timeout: DurationMillis
) -> Option<(ButtonEvent,ButtonState)> {
    loop {
        let ev = ber.read_event();
        match ev {
            ButtonEvent::None => {},
            ev_not_none => {
                return Some((ev_not_none,ber.state()));
            }
        }
        super::wait_event_timeout(sys, SignalFlags::Button);
    }
}
*/
