use super::{AccessTiming, DurationMillis, InstantMillis};
use crate::sleep_at_most;

/// useful for waiting until a certain time has passed or an event has occured
pub struct Timer<'a, S> {
    start_time: InstantMillis,
    duration: DurationMillis,
    sys: &'a S,
}

impl<'a, S: AccessTiming> Timer<'a, S> {
    /// creates a timer object for waiting the specified `duration` in milliseconds
    pub fn new(sys: &'a S, duration: DurationMillis) -> Self {
        Self {
            start_time: sys.get_instant(),
            duration,
            sys,
        }
    }
    /// time left
    ///
    /// The calculation is based on the values
    /// returned by [`AccessTiming::get_instant`]
    /// and the requested duration.
    ///
    /// If the requested duration has been
    /// surpassed, the return value becomes negative.  
    pub fn time_left(&self) -> DurationMillis {
        let time_passed = self.sys.get_instant() - self.start_time;
        self.duration - time_passed
    }
    /// Returns `false` if no time is left, otherwise
    /// requests a delay that lasts at most for the remaining time
    /// and returns controll to the caller.
    pub async fn yield_if_time_left(&self) -> bool {
        let time_left = self.time_left();
        if time_left <= 0 {
            return false;
        }
        sleep_at_most(self.sys, time_left).await;
        true
    }
    /// waits for the remaining duration
    ///
    /// This function does not permit
    /// stopping the timer early. If
    /// this is needed,
    /// create a loop an use [`Self::yield_if_time_left]
    /// instead.
    pub async fn wait(&self) {
        while self.yield_if_time_left().await {}
    }
    /// returns the instant when [`Self::new`] was invoked.
    pub fn start_time(&self) -> InstantMillis {
        self.start_time
    }
}

/*

pub async fn wait_cond(cond: impl Fn()->bool) {
    while !cond() {
        yield_now().await;
    }
}


pub async fn delay(scheduler: &impl AccessTiming, timeout: DurationMillis) {
    let start_time = scheduler.get_instant();
    scheduler.set_delay_request(timeout);
    loop{
        yield_now().await;
        let time_passed = scheduler.get_instant() - start_time;
        let time_left = timeout - time_passed;
        if time_left <= 0 {
            break;
        }
        scheduler.set_delay_request(time_left);
    }
}

/// wait for condition to become true or timeout
///
/// returns true iff the `cond()` evaluates to true
pub async fn wait_cond_timeout(scheduler: &impl AccessTiming, timeout: DurationMillis, cond: impl Fn()->bool) -> bool {
    if cond() {
        return true;
    }
    let start_time = scheduler.get_instant();
    scheduler.set_delay_request(timeout);
    loop{
        yield_now().await;
        if cond() {
            return true;
        }
        let time_passed = scheduler.get_instant() - start_time;
        let time_left = timeout - time_passed;
        if time_left <= 0 {
            return false;
        }
        scheduler.set_delay_request(time_left);
    }
}

/// evaluate closure until it does not evaluate to `None` or until timeout
///
/// returns the result of the closure
pub async fn wait_avail<T>(f: impl Fn()->Option<T>) -> T {
    loop {
        if let Some(res) = f() {
            return res;
        }
        yield_now().await;
    }
}


/// evaluate closure until it does not evaluate to `None` or until timeout
///
/// returns the result of the closure
pub async fn wait_avail_timeout<T>(scheduler: &impl AccessTiming, timeout: DurationMillis, f: impl Fn()->Option<T>) -> Option<T> {
    if let Some(res) = f() {
        return Some(res);
    }
    let start_time = scheduler.get_instant();
    scheduler.set_delay_request(timeout);
    loop{
        yield_now().await;
        if let Some(res) = f() {
            return Some(res);
        }
        let time_passed = scheduler.get_instant() - start_time;
        let time_left = timeout - time_passed;
        if time_left <= 0 {
            return None;
        }
        scheduler.set_delay_request(time_left);
    }
}

*/
