//! simple executor to be integrated into the event loop of an embedded device

// no_std only when freature "std" is missing
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use ambassador::delegatable_trait;
use core::cell::Cell;

mod basic_futures;
mod executor;
mod milliseconds;
mod timer;

pub use basic_futures::{join2, yield_now};
pub use milliseconds::{DurationMillis, InstantMillis};

//use crate::system::SignalFlags;

/*
pub use futures::{
    delay,
    wait_cond,
    wait_cond_timeout,
    wait_avail,
    wait_avail_timeout
};
*/
pub use executor::{MiniExecutor, MiniExecutorBuilder, MiniScheduler};
pub use timer::Timer;

/// for internal use
///
/// This wrapper cannot be created outside this module.
/// It protects [`AccessTiming::set_delay_request`] from
/// being used outside this module.
///
pub struct DurationWrapper(DurationMillis);

/// access to the timing of execution
#[delegatable_trait]
pub trait AccessTiming {
    /// returns the instant passed to the latest call to [`executor::MiniExecutor::step`]
    fn get_instant(&self) -> InstantMillis;
    /// Use `sleep_at_most()` instead. This function is internal to this module.
    fn set_delay_request(&self, millis: DurationWrapper);
}

/// interrupts execution and resumes after at most the given duration    
pub async fn sleep_at_most(sys: &impl AccessTiming, duration: DurationMillis) {
    sys.set_delay_request(DurationWrapper(duration));
    yield_now().await;
}

/// access to the signals passed to [`executor::MiniExecutor::step`]
#[delegatable_trait]
pub trait AccessExecutorSignals<X> {
    /// get the data that was passed to the latest call to [`executor::MiniExecutor::step`]
    fn get_executor_signals(&self) -> X;
}

/// trait for a "scheduler" which manages timing of the calls to the `step()` function
///
/// `X` is the type for executor signals and needs to implement `Copy` and `Default`,
/// see documentation of [`MiniExecutor::step`]
pub trait Scheduler<X>: AccessTiming + AccessExecutorSignals<X> {}
impl<T, X> Scheduler<X> for T where T: AccessTiming + AccessExecutorSignals<X> {}

/// An struct that can be used as an error return value when a stop signal was received
pub struct Stopped;

/// an abstraction useful for boolean signals
pub trait ReceiveBool {
    fn get(&self) -> bool;
}

impl ReceiveBool for Cell<bool> {
    fn get(&self) -> bool {
        self.get()
    }
}
