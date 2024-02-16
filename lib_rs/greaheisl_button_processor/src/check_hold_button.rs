use greaheisl_async::{sleep_at_most, AccessTiming};
use greaheisl_async::{DurationMillis, InstantMillis};

use super::{ButtonEvent, ButtonFlagsTrait, ButtonProcessor, ButtonState};

pub enum CheckHoldButtonResult {
    /// button state has changed to something else; no interpretable event
    Other,
    /// button(s) were released early
    ReleaseEarly,
    /// buttons have been held for the required duration
    Hold,
}

/// helps to watch if the user has been holding down a button for a specified amount of time
pub struct CheckHoldButton<'a, S, F> {
    duration: DurationMillis,
    sys: &'a S,
    btnp: &'a ButtonProcessor<F>,
    since: InstantMillis,
    button_flags: F,
}

impl<'a, S: AccessTiming, F: ButtonFlagsTrait> CheckHoldButton<'a, S, F> {
    /// initializes the `CheckHoldButton` structure
    ///
    /// Prerequisite for this function is that the user is already
    /// pressing the button combination we are looking for.
    /// If no buttons are held down when this function is called,
    /// a panic is raised.
    ///
    /// The`duration` specifies the time in milliseconds that
    /// the user needs to hold down the button combination.
    pub fn new(sys: &'a S, btnp: &'a ButtonProcessor<F>, duration: DurationMillis) -> Self {
        let ButtonState::SomeButtons {
            button_flags,
            since,
            last_repetition: _,
        } = btnp.state()
        else {
            panic!("No button pressed. Cannot wait for holding it.");
        };
        Self {
            since,
            duration,
            sys,
            btnp,
            button_flags,
        }
    }
    /// calculates the time left until the user has
    /// held down the button for the given duration
    ///
    /// Can be negative.
    pub fn time_left(&self) -> DurationMillis {
        let time_passed = self.sys.get_instant() - self.since;
        self.duration - time_passed
    }
    /// Returns with some [`CheckHoldButtonResult`] if the button state has
    /// changed or if time is up, otherwise
    /// requests a delay that lasts at most for the remaining time
    /// and falls asleep for some time. In that case the return value is `None`.
    ///
    /// You can use this function if you need to watch for events other
    /// than the button combination being held down.
    /// Otherwise, [`CheckHoldButton::wait`] is simpler to use.
    pub async fn yield_if_time_left(&self) -> Option<CheckHoldButtonResult> {
        match self.btnp.event() {
            ButtonEvent::Release(bflg) if bflg == self.button_flags => {
                return Some(CheckHoldButtonResult::ReleaseEarly)
            }
            _ => {}
        };
        let ButtonState::SomeButtons {
            button_flags,
            since: _,
            last_repetition: _,
        } = self.btnp.state()
        else {
            return Some(CheckHoldButtonResult::Other);
        };
        if button_flags != self.button_flags {
            return Some(CheckHoldButtonResult::Other);
        }
        let time_left = self.time_left();
        if time_left <= 0 {
            return Some(CheckHoldButtonResult::Hold);
        }
        sleep_at_most(self.sys, time_left).await;
        None
    }
    /// waits until either
    /// *   the button combination has been held down for the specified duration, or
    /// *   the button state changes early
    pub async fn wait(&self) -> CheckHoldButtonResult {
        loop {
            if let Some(result) = self.yield_if_time_left().await {
                return result;
            };
        }
    }
    /// get the instant since when the button combination is held down, in milliseconds
    pub fn start_time(&self) -> InstantMillis {
        self.since
    }
}
