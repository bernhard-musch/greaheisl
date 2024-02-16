use super::DurationWrapper;
use super::{AccessExecutorSignals, AccessTiming};
use super::{DurationMillis, InstantMillis};
use alloc::boxed::Box;
use alloc::rc::Rc;
use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;
use core::ptr;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

pub struct MiniExecutorBuilder<X> {
    scheduler: Rc<RefCell<MiniScheduler<X>>>,
}

/// simple executor to be integrated into the event loop of an embedded device
///
/// You need to call the [`MiniExecutor::step`] function
/// to run the single task managed by the executor.
/// This task can fork into emulated parallel tasks
/// using [`crate::basic_futures::join2()`].
///
pub struct MiniExecutor<X> {
    task: Option<Pin<Box<dyn Future<Output = ()>>>>,
    waker: Waker,
    scheduler: Rc<RefCell<MiniScheduler<X>>>,
}

/// an implementation of the [`super::Scheduler`] trait for [`MiniExecutor`]
pub struct MiniScheduler<X> {
    delay_request: Option<DurationMillis>,
    instant: InstantMillis,
    executor_signals: X,
}

impl<X> AccessTiming for Rc<RefCell<MiniScheduler<X>>> {
    fn set_delay_request(&self, delay: DurationWrapper) {
        let delay_mut = &mut self.borrow_mut().delay_request;
        if let Some(ref mut delay_mut) = delay_mut {
            if *delay_mut > delay.0 {
                *delay_mut = delay.0;
            }
        } else {
            *delay_mut = Some(delay.0);
        }
    }

    fn get_instant(&self) -> InstantMillis {
        self.borrow().instant
    }
}

impl<X: Copy> AccessExecutorSignals<X> for Rc<RefCell<MiniScheduler<X>>> {
    fn get_executor_signals(&self) -> X {
        self.borrow().executor_signals
    }
}

static VTABLE: RawWakerVTable = RawWakerVTable::new(
    |_| RawWaker::new(ptr::null(), &VTABLE),
    |_| {},
    |_| {},
    |_| {},
);

impl<X> MiniExecutorBuilder<X> {
    /// You can use this to obtain a clone of the scheduler and pass it to the
    /// future representing the main task.
    pub fn scheduler(&self) -> &Rc<RefCell<MiniScheduler<X>>> {
        &self.scheduler
    }

    /// second stage of initializing the executor
    ///
    /// `fut` is the main task run by the executor.
    ///  Returns an instance of the `MiniExecutor`.
    pub fn build(self, fut: impl Future<Output = ()> + 'static) -> MiniExecutor<X> {
        let raw_waker = RawWaker::new(ptr::null(), &VTABLE);
        let waker = unsafe { Waker::from_raw(raw_waker) };
        let task = Box::pin(fut);
        MiniExecutor {
            task: Some(task),
            waker,
            scheduler: self.scheduler,
        }
    }
}

impl<X: Copy + Default> MiniExecutor<X> {
    /// first stage of initialization of the executor
    ///
    /// `start_time` needs to be the current time when the executor is created.
    /// Returns an executor builder, needed for the second stage of initialization.
    pub fn new(start_time: InstantMillis) -> MiniExecutorBuilder<X> {
        let scheduler = Rc::new(RefCell::new(MiniScheduler {
            delay_request: None,
            instant: start_time,
            executor_signals: X::default(),
        }));
        MiniExecutorBuilder { scheduler }
    }

    /// runs the `poll()` function of the main task once
    ///
    /// `instant` is the current time with milli second resolution.
    /// `executor signals` can be any type of data implementing `Copy` and `Default`.
    /// This information can be made available to the running tasks
    /// by means of the scheduler. It can be used to indicate
    /// whether any events have happened and triggered the call to `step()`.
    ///
    /// The return value is `Some(duration)` in milliseconds if the running task
    /// has called [`AccessTiming::set_delay_request`] at least once in its `poll()` function.
    /// The duration is the requested maximum delay before the next call to `step()`.
    /// Calling `step()` sooner is always acceptable, and
    /// may be needed if an event happened that requires immediate processing.
    ///
    /// The return value `None` indicates that [`AccessTiming::set_delay_request`]
    /// has not been called. In that case, the next call to `step()` needs to be
    /// made when an event has happened, at the latest.
    pub fn step(&mut self, instant: InstantMillis, executor_signals: X) -> Option<DurationMillis> {
        {
            let mut scheduler = self.scheduler.borrow_mut();
            scheduler.instant = instant;
            scheduler.executor_signals = executor_signals;
            scheduler.delay_request = None;
        }
        let mut context = Context::from_waker(&self.waker);
        let Some(ref mut task) = self.task else {
            return None;
        };
        let pollres = task.as_mut().poll(&mut context);
        if let Poll::Ready(()) = pollres {
            self.task = None;
            return None;
        } else {
            let Some(delay_request) = self.scheduler.borrow_mut().delay_request else {
                return Some(0);
            };
            return Some(delay_request);
        }
    }
}
