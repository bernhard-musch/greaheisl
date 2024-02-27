//! simple executor to be integrated into the event loop of an embedded device
//!
//! ## example
//!
//! ```
//! use greaheisl_async::{join2, sleep_at_most, AccessTiming, InstantMillis, MiniExecutor};
//! use std::rc::Rc;
//! use std::cell::RefCell;
//! 
//! async fn my_main_task(sys: impl AccessTiming, result: Rc<RefCell<Vec<i32>>>) {
//!     result.borrow_mut().push(-1);
//!     sleep_at_most(&sys,100).await;
//!     result.borrow_mut().push(-2);
//!     sleep_at_most(&sys,100).await;
//!     result.borrow_mut().push(-3);
//!     sleep_at_most(&sys,100).await;
//!     result.borrow_mut().push(-4);
//!     sleep_at_most(&sys,100).await; 
//!     // start executing two tasks quasi-parallel
//!     join2(async {
//!         result.borrow_mut().push(-10);
//!         sleep_at_most(&sys,100).await;
//!         result.borrow_mut().push(-20);
//!         sleep_at_most(&sys,100).await;
//!     }, async {
//!         result.borrow_mut().push(-100);
//!         // The shorter sleep time will win!
//!         sleep_at_most(&sys,99).await;
//!         result.borrow_mut().push(-200);
//!         sleep_at_most(&sys,99).await;
//!     })
//!     .await;
//! }
//! 
//! 
//! let mut time = InstantMillis::from_absolute(10);
//! let executor_builder = MiniExecutor::new(time);
//! let sys = executor_builder.scheduler().clone();
//! let result = Rc::new(RefCell::new(Vec::new()));
//! let mut executor = executor_builder.build(my_main_task(sys,result.clone()));
//! result.borrow_mut().push(time.into_inner() as i32);
//! time += 10;
//! for _i in 0..6 {
//!     //! repeatedly call the `executor.step()` function
//!     if let Some(max_wait) = executor.step(time,()) {
//!         time += max_wait;
//!         result.borrow_mut().push(time.into_inner() as i32);
//!     }
//! }
//! let result = result.borrow().clone();
//! assert_eq!(result,vec![10, -1, 120, -2, 220, -3, 320, -4, 420, -10, -100, 519, -20, -200, 618])
//! ```
//!
//! ## Features
//!
//! - `std`: (default) uses standard library.
//!    *Note:* set `default-features = false` for no-std targets.

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
