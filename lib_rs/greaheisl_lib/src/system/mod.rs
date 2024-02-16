//! Module with hardware related functionality

use bitmask_enum::bitmask;
use greaheisl_async::DurationMillis;
use greaheisl_async::Scheduler;
use greaheisl_async::{
    sleep_at_most, yield_now, AccessExecutorSignals, AccessTiming, ReceiveBool, Stopped,
};
use greaheisl_button_processor::{AccessButtonSignal, AccessButtonState};

pub mod buttons;

/// time of day in human readable notation, with second resolution
#[repr(C)]
pub struct RtcTime {
    /// valid numbers restricted to 0..23
    pub hour: u8,
    /// valid numbers restricted to 0..59
    pub minute: u8,
    /// valid numbers restricted to 0..59
    pub second: u8,
}

/// time of day from the real time clock
pub trait AccessRtc {
    fn get_rtc(&self) -> RtcTime;
    fn set_rtc(&self, time: &RtcTime);
}

/// the built-in 12 by 8 LED matrix
pub trait AccessLedMatrix {
    fn set_led_matrix(&self, matrix: &[u32; 3]);
}

/// number of output relays
pub const NUM_RELAYS: usize = 4;

/// to switch on the device connected to the relay,
/// set the corresponding relay state to true
pub trait AccessOutputStates {
    fn set_relay_states(&self, relais_states: &[bool; NUM_RELAYS]);
}

/// All the functionality provided by means of callbacks
///
/// Note that you do not need to implement the trait `Callbacks` explicitly,
/// thanks to the blanket implementation.
pub trait Callbacks:
    AccessButtonState<ButtonFlags = buttons::ButtonFlags>
    + AccessRtc
    + AccessLedMatrix
    + AccessOutputStates
{
}
impl<T> Callbacks for T where
    T: AccessButtonState<ButtonFlags = buttons::ButtonFlags>
        + AccessRtc
        + AccessLedMatrix
        + AccessOutputStates
{
}

/// All the system functionality united in one trait
pub trait System: Callbacks + Scheduler<SignalFlags> + AccessButtonSignal {}
impl<T> System for T where T: Scheduler<SignalFlags> + Callbacks + AccessButtonSignal {}

/// Right now we only support one kind of event
///
/// Note that we allow spurious events.
/// For example, it is not a problem if the `Button` flag is set even though
/// there is no change in button states.
/// On the other hand, it is not OK if the physical button state changes
/// but the `Button` flat is not set. In that case, the change
/// in state may be overlooked.
#[bitmask(u8)]
pub enum SignalFlags {
    /// set this flag if the state of any of the buttons changes
    Button,
}

impl Default for SignalFlags {
    fn default() -> Self {
        SignalFlags::none()
    }
}

/// Waits until one of the set bits in `event_signal`
/// matches with one of the set bits in the singal flags.
/// Returns the matching signal flags.
pub async fn wait_event(
    sys: &impl AccessExecutorSignals<SignalFlags>,
    event_signal: SignalFlags,
) -> SignalFlags {
    loop {
        yield_now().await;
        let sig = sys.get_executor_signals();
        let inters = sig & event_signal;
        if inters != 0 {
            return inters;
        }
    }
}

/// Waits until one of the set bits in `event_signal`
/// matches with one of the set bits in the singal flags.
/// In this case the funtion returns `Ok` with the matching
/// signal flags.
/// Also stops waiting if the stop signal becomes true.
/// In that case the function returns `Err(Stopped)`.
pub async fn wait_stop_or_event(
    sys: &impl AccessExecutorSignals<SignalFlags>,
    stop_signal: &impl ReceiveBool,
    event_signal: SignalFlags,
) -> Result<SignalFlags, Stopped> {
    while !stop_signal.get() {
        yield_now().await;
        let sig = sys.get_executor_signals();
        let inters = sig & event_signal;
        if inters != 0 {
            return Ok(inters);
        }
    }
    Err(Stopped)
}

/// Waits until one of the set bits in `event_signal`
/// matches with one of the set bits in the singal flags.
/// In this case the funtion returns `Ok` with the matching
/// signal flags.
/// Also stops waiting if the stop signal becomes true.
/// In that case the function returns `Err(Stopped)`.
/// Also stops waiting after the specified timeout.
/// In that case the function returns `Ok(SignalFlags::None)`.
pub async fn wait_stop_or_event_timeout(
    sys: &(impl AccessTiming + AccessExecutorSignals<SignalFlags>),
    stop_signal: &impl ReceiveBool,
    event_signal: SignalFlags,
    timeout: DurationMillis,
) -> Result<SignalFlags, Stopped> {
    let start_time = sys.get_instant();
    let mut time_left = timeout;
    while !stop_signal.get() {
        sleep_at_most(sys, time_left).await;
        let sig = sys.get_executor_signals();
        let inters = sig & event_signal;
        if inters != 0 {
            return Ok(inters);
        }
        let time_passed = sys.get_instant() - start_time;
        time_left = timeout - time_passed;
        if time_left <= 0 {
            return Ok(SignalFlags::none());
        }
    }
    Err(Stopped)
}
