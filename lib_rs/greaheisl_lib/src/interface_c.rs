//! definition of the foreign function interface of this library

//use static_assertions::const_assert_eq;
use crate::system::buttons;
use crate::system::{
    AccessLedMatrix, AccessOutputStates, AccessRtc, RtcTime, SignalFlags, NUM_RELAYS,
};
use alloc::boxed::Box;
#[cfg(any(not(feature = "std"),doc))]
use core::ffi::c_void;
use core::mem::MaybeUninit;
use greaheisl_button_processor::{AccessButtonSignal, AccessButtonState};
//use crate::imagematrix::ImageMatrixSliceMut;
use crate::run;
use ambassador::Delegate;
use greaheisl_async::{
    ambassador_impl_AccessExecutorSignals, ambassador_impl_AccessTiming, AccessExecutorSignals,
    AccessTiming, DurationWrapper, MiniExecutor,
};
use greaheisl_async::{DurationMillis, InstantMillis};

/// the callback functions that the outer framework needs to provide to us
#[derive(Clone)]
#[repr(C)]
pub struct GreaheislCallbacks {
    /// provides hour, minute and second
    pub get_rtc: Option<unsafe extern "C" fn(*mut RtcTime)>,
    /// sets hour, minute and second
    pub set_rtc: Option<unsafe extern "C" fn(*const RtcTime)>,
    /// sets the bitmap shown by the LED matrix
    pub set_led_matrix: Option<unsafe extern "C" fn(&[u32; 3])>,
    /// returns the states of the buttons as binary flags
    pub get_button_flags: Option<unsafe extern "C" fn() -> u8>,
    /// sets the relay states (on or off)
    pub set_relay_states: Option<unsafe extern "C" fn(&[bool; NUM_RELAYS])>,
}

#[derive(Delegate)]
#[delegate(AccessTiming, target = "scheduler")]
#[delegate(AccessExecutorSignals<SignalFlags>,target = "scheduler")]
struct CSystem<S> {
    callbacks: &'static GreaheislCallbacks,
    scheduler: S,
}

impl<S> AccessRtc for CSystem<S> {
    fn get_rtc(&self) -> RtcTime {
        let mut rtc_time = MaybeUninit::<RtcTime>::uninit();
        unsafe { (self.callbacks.get_rtc.unwrap())(rtc_time.as_mut_ptr()) };
        unsafe { rtc_time.assume_init() }
    }

    fn set_rtc(&self, time: &RtcTime) {
        unsafe { (self.callbacks.set_rtc.unwrap())(time) }
    }
}

impl<S> AccessLedMatrix for CSystem<S> {
    fn set_led_matrix(&self, matrix: &[u32; 3]) {
        unsafe { (self.callbacks.set_led_matrix.unwrap())(matrix) };
    }
}

impl<S> AccessButtonState for CSystem<S> {
    type ButtonFlags = crate::system::buttons::ButtonFlags;
    fn get_button_flags(&self) -> buttons::ButtonFlags {
        let bf = unsafe { (self.callbacks.get_button_flags.unwrap())() };
        buttons::ButtonFlags::from(bf)
    }
}

impl<S: AccessExecutorSignals<SignalFlags>> AccessButtonSignal for CSystem<S> {
    fn is_button_signal(&self) -> bool {
        self.get_executor_signals().contains(SignalFlags::Button)
    }
}

impl<S> AccessOutputStates for CSystem<S> {
    fn set_relay_states(&self, relay_states: &[bool; crate::system::NUM_RELAYS]) {
        unsafe { (self.callbacks.set_relay_states.unwrap())(relay_states) }
    }
}

/*
fn get_event(&self) -> ui::ButtonEvent {
    unsafe{ (self.get_event.unwrap())() }
}
*/

/// obtains function handles for memory allocation from the outer framework
///
/// ## aligned alloc:
/// callback funtion to allocate a block of memory;
///
/// arguments of the function handle
/// * first arguments: requested amount of bytes
/// * second argument: requested block alignment, in bytes
///
/// returns a pointer to the allocated memory, or `null` upon failure
///
/// ## free:
/// callback function to free a block of memory
///
/// The block must have been previously allocated using the
/// callback function registered though the `aligned_alloc` member.
/// It is also allowed to pass `null` to the function,
/// but in that case it does nothing.

// rerouting allocator only needed when we build for embedded device
#[cfg(any(not(feature = "std"),doc))]
#[no_mangle]
pub unsafe extern "C" fn set_allocator_functions(
    aligned_alloc: Option<unsafe extern "C" fn(usize, usize) -> *mut c_void>,
    free: Option<unsafe extern "C" fn(*mut c_void)>,
) {
    crate::delegating_alloc::init_delegating_allocatator(aligned_alloc, free)
}

/// the executor provided to the outer framework
pub type GreaheislExecutor = MiniExecutor<SignalFlags>;


/// initialization of the executor
///
/// [`set_allocator_functions`] must be called *before* this function is called. 
///
/// arguments:
/// * `callbacks`: a structure of with valid function pointers to the callback functions
/// * `instant`: the current time in milliseconds (with an arbitrary offset)
///
/// Returns a pointer to the executor. Note that the executor
/// is allocated on the heap. 
#[no_mangle]
pub extern "C" fn greaheisl_init(
    callbacks: &'static GreaheislCallbacks,
    instant: u32,
) -> *mut GreaheislExecutor {
    let gh = MiniExecutor::new(InstantMillis::from_absolute(instant));
    let sys = CSystem {
        callbacks,
        scheduler: gh.scheduler().clone(),
    };
    let task = run(sys);
    let gh = gh.build(task);
    Box::into_raw(Box::new(gh))
}

/// lets the executor perform one step
///
/// arguments:
/// * `handle`: pointer to the executor, as obtained from [`greaheisl_init`]
/// * `instant`: the time in milliseconds
/// * `signals`: singal flags, indicating what kind of event has happened.
///   Note that spurious signals are allowed, but neglected event signals are not.
///
/// returns the number of milliseconds allowed to delay
/// the next call of `greaheisl_step()`, unless an event 
/// happens before
#[no_mangle]
pub extern "C" fn greaheisl_step(
    handle: &mut GreaheislExecutor,
    instant: u32,
    signals: u8,
) -> DurationMillis {
    let signals = SignalFlags::from(signals);
    if let Some(delay_request) = handle.step(InstantMillis::from_absolute(instant), signals) {
        delay_request
    } else {
        DurationMillis::MAX
    }
}

/*
#[no_mangle]
pub extern "C" fn show_clock(imat: *mut u32, hours: u8, minutes: u8) {
    let imat = unsafe{ core::slice::from_raw_parts_mut(imat, 3) };
    let mut imat = ImageMatrixSliceMut::<12,8>(imat);
    ui::display_clock(&mut imat,hours,minutes);
}
*/
