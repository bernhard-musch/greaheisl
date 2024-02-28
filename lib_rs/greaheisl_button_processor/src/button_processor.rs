use core::{cell::Cell, future::Future};

use greaheisl_async::{join2, yield_now};
use greaheisl_async::{sleep_at_most, AccessTiming};
use greaheisl_async::{DurationMillis, InstantMillis};
use greaheisl_async::{ReceiveBool, Stopped};

use super::{AccessButtonSignal, AccessButtonState};

/// The events distinguished by the high level button processor
///
/// The generic parameter `F` represents the "button flags",
/// i.e., the raw on/off state of each button.
#[derive(Copy, Clone, Debug)]
pub enum ButtonEvent<F> {
    None,
    /// button (combination) pressed for the first time
    Press(F),
    /// button (combination) auto repeat event due to held button(s)
    Repeat(F),
    /// button (combination) released
    Release(F),
}

/// state memory of the high level butotn processor
///
/// The generic parameter `F` represents the "button flags",
/// i.e., the raw on/off state of each button.
#[derive(Copy, Clone, Debug)]
pub enum ButtonState<F> {
    /// Certain button state transitions have no valid interpretation.
    /// Suppose several buttons have have been pressed and now
    /// one is released. The state afterwards is `Invalid` and remains
    /// so until all buttons have been released and the state
    /// returns to `NoButtons`.  
    Invalid,
    /// No buttons are pressed.
    NoButtons,
    /// Some buttons have been pressed.
    SomeButtons {
        /// the combination of buttons pressed
        button_flags: F,
        /// the instant since when this combination is present
        since: InstantMillis,
        /// the last time a `Repeat` event was issued
        last_repetition: Option<InstantMillis>,
    },
}

pub trait ButtonFlagsTrait: Copy + PartialEq {
    fn is_none(&self) -> bool;
    fn contains(&self, other: Self) -> bool;
}

#[derive(Clone, Debug)]
pub struct ButtonProcessorOptions {
    /// after this many milliseconds, a first `Repeat` event is generated
    pub repetition_start_delay: DurationMillis,
    /// after this many milliseconds, another `Repeat` event is generated
    pub repetition_delay: DurationMillis,
}

impl core::default::Default for ButtonProcessorOptions {
    fn default() -> Self {
        Self {
            repetition_start_delay: 750,
            repetition_delay: 375,
        }
    }
}

/// button processor
///
/// From the hardware, we get an on/off state for each of the buttons,
/// encoded in "button flags" of generic type `F`.
/// Building your logic on these raw button states is possible but
/// cumbersome. The button processor helps you here. It recognizes
/// state changes and turns turns them into "events".
///
/// *   When one or several buttons are pushed simultaneously,
///     a [`ButtonEvent::Press`] event is issued.
/// *   When additional buttons are pushed, another
///     [`ButtonEvent::Press`] event is issued.
///     This behavior allows you to respond to button
///     combinations.
///     In reality, it is hard for the user to push buttons
///     at once deliberately.
///     Typically, pushing several buttons results in
///     multiple state changes. You need to take this
///     into account. If you want to wait for a
///     button combination, you need to decide
///     which sequences of button states can lead up
///     to the desired combination. You need to ignore the
///     respective [`ButtonEvent::Press`] events.
/// *   When a button combination is held down
///     for longer than [`ButtonProcessorOptions::repetition_start_delay`] milliseconds,
///     a first [`ButtonEvent::Repeat`] event is issued.
///     Further [`ButtonEvent::Repeat`] events raised at
///     a shorter time period, specified by [`ButtonProcessorOptions::repetition_delay`]
///     in milliseconds.
/// *   When one or several buttons are released,
///     a [`ButtonEvent::Release`] event is issued.
///     Further button state
///     changes are ignored until all buttons
///     are released.
///     This behavior helps you to filter out
///     state changes that do not correspond
///     to typical usage patterns.
///     In situations where you want to be able
///     to react to button combinations,
///     long button push times, etc., and yet
///     be able to respond to single button
///     events, it can be useful to wait for the
///     [`ButtonEvent::Release`] events of single buttons
///     insteaad of the corresponding [`ButtonEvent::Press`] events.
pub struct ButtonProcessor<F> {
    event: Cell<ButtonEvent<F>>,
    state: Cell<ButtonState<F>>,
    options: ButtonProcessorOptions,
}

impl<F: ButtonFlagsTrait> ButtonProcessor<F> {
    /// creates the button processor
    pub fn new(options: ButtonProcessorOptions) -> Self {
        ButtonProcessor {
            event: Cell::new(ButtonEvent::None),
            state: Cell::new(ButtonState::Invalid),
            options,
        }
    }
    /// gets the current high level event, or `ButtonEvent::None` if there is none
    ///
    /// There is no event queue.
    /// Normally, you will want to check for an event each time the `poll()` function
    /// of the main task is called.
    /// In other words, you need to invoke this function after each
    /// call to an `async` function of the `yield_XXX` kind.
    ///
    /// If you do not check every time, then events can get "lost",
    /// i.e., there is no reaction to them.
    pub fn event(&self) -> ButtonEvent<F> {
        self.event.get().clone()
    }
    /// get information about the state of the button processor
    ///
    /// You can use this function to get information about the
    /// button state inbetween events.
    /// For example, this can be useful if you want to react when the
    /// user holds down a button combination longer
    /// than a certain time.
    pub fn state(&self) -> ButtonState<F> {
        self.state.get()
    }
    /// Runs the button processor.
    ///
    /// -   The button processor requires low level access to the system through `sys`.
    /// -   `fut` is your task that needs the button processor. From within `fut`, use a reference
    ///     to the button processor and the the functions [`ButtonProcessor::event`],
    ///     [`ButtonProcessor::state`] to react to button events.`
    pub async fn run<T>(
        &self,
        sys: &(impl AccessButtonState<ButtonFlags = F> + AccessButtonSignal + AccessTiming),
        fut: impl Future<Output = T>,
    ) -> T {
        let stop_signal = Cell::new(false);
        let driver = self.button_processor_task(sys, &stop_signal);
        join2(driver, async {
            let retval = fut.await;
            stop_signal.set(true);
            retval
        })
        .await
        .1
    }
    async fn button_processor_task(
        &self,
        sys: &(impl AccessTiming + AccessButtonSignal + AccessButtonState<ButtonFlags = F>),
        stop_signal: &impl ReceiveBool,
    ) {
        while !stop_signal.get() {
            self.event.set(ButtonEvent::None);
            let res = self.button_processor_step(sys, stop_signal).await;
            if let Err(Stopped) = res {
                break;
            }
        }
    }
    async fn button_processor_step(
        &self,
        sys: &(impl AccessTiming + AccessButtonSignal + AccessButtonState<ButtonFlags = F>),
        stop_signal: &impl ReceiveBool,
    ) -> Result<(), Stopped> {
        //const BF_NONE : F = F::default();
        let current_flags = sys.get_button_flags();
        match self.state.get() {
            ButtonState::Invalid => {
                // transition from uninitialized or invalid state to state with no buttons down
                if current_flags.is_none() {
                    self.state.set(ButtonState::NoButtons);
                }
                wait_stop_or_button(sys, stop_signal).await?;
                // Note that we ignore all transitions to states with buttons pressed.
                // After an invalid state, we always have to go through "no buttons down".
            }
            ButtonState::NoButtons => {
                if current_flags.is_none() {
                    // still no activity
                    wait_stop_or_button(sys, stop_signal).await?;
                } else {
                    // transition from no buttons pressed to some buttons pressed
                    self.event.set(ButtonEvent::Press(current_flags));
                    self.state.set(ButtonState::SomeButtons {
                        button_flags: current_flags,
                        since: sys.get_instant(),
                        last_repetition: None,
                    });
                    // Now it's important not to use one of the `wait_` routines,
                    // because we immediately need to go back to `ButtonEvent::None` next time we get polled!
                    sleep_at_most(sys, self.options.repetition_start_delay).await;
                }
            }
            ButtonState::SomeButtons {
                button_flags: prev_flags,
                since,
                last_repetition,
            } => {
                match current_flags {
                    a_flags if a_flags == prev_flags => {
                        // buttons unchanged;
                        // we may need to trigger a repetition event
                        let now = sys.get_instant();
                        if let Some(repetition_instant) = last_repetition {
                            let time_until_firing =
                                self.options.repetition_delay - (now - repetition_instant);
                            // we are already firing repeatedly
                            if time_until_firing <= 0 {
                                // fire once more
                                self.event.set(ButtonEvent::Repeat(current_flags));
                                self.state.set(ButtonState::SomeButtons {
                                    button_flags: current_flags,
                                    since,
                                    last_repetition: Some(now),
                                });
                                // Now it's important not to use one of the `wait_` routines,
                                // because we immediately need to go back to `ButtonEvent::None` next time we get polled!
                                sleep_at_most(sys, self.options.repetition_delay).await;
                            } else {
                                // fire not just yet
                                wait_stop_or_button_or_timeout(sys, stop_signal, time_until_firing)
                                    .await?;
                            }
                        } else {
                            // so far no repeated firing
                            let time_until_repeating =
                                self.options.repetition_start_delay - (now - since);
                            if time_until_repeating <= 0 {
                                // start repeated firing
                                self.event.set(ButtonEvent::Repeat(current_flags));
                                self.state.set(ButtonState::SomeButtons {
                                    button_flags: current_flags,
                                    since,
                                    last_repetition: Some(now),
                                });
                                // Now it's important not to use one of the `wait_` routines,
                                // because we immediately need to go back to `ButtonEvent::None` next time we get polled!
                                sleep_at_most(sys, self.options.repetition_delay).await;
                            } else {
                                // fire not just yet
                                wait_stop_or_button_or_timeout(
                                    sys,
                                    stop_signal,
                                    time_until_repeating,
                                )
                                .await?;
                            }
                        }
                    }
                    a_flags if a_flags.contains(prev_flags) => {
                        // more buttons than before; thats a valid transition, so we produce a `Press` event
                        self.event.set(ButtonEvent::Press(current_flags));
                        self.state.set(ButtonState::SomeButtons {
                            button_flags: current_flags,
                            since: sys.get_instant(),
                            last_repetition: None,
                        });
                        // Now it's important not to use one of the `wait_` routines,
                        // because we immediately need to go back to `ButtonEvent::None` next time we get polled!
                        sleep_at_most(sys, self.options.repetition_start_delay).await;
                    }
                    _x if _x.is_none() => {
                        // transition from uninitialized or invalid state to state with no buttons down
                        self.event.set(ButtonEvent::Release(prev_flags));
                        self.state.set(ButtonState::NoButtons);
                        // Now it's important not to use one of the `wait_` routines,
                        // because we immediately need to go back to `ButtonEvent::None` next time we get polled!
                        yield_now().await;
                    }
                    _ => {
                        // some buttons were released;
                        // the state after that is invalid until all buttons have been released
                        self.event.set(ButtonEvent::Release(prev_flags));
                        self.state.set(ButtonState::Invalid);
                        // Now it's important not to use one of the `wait_` routines,
                        // because we immediately need to go back to `ButtonEvent::None` next time we get polled!
                        yield_now().await;
                    }
                }
            }
        }
        Ok(())
    }
}

/// Waits until one of the set bits in `event_signal`
/// matches with one of the set bits in the singal flags.
/// In this case the funtion returns `Ok` with the matching
/// signal flags.
/// Also stops waiting if the stop signal becomes true.
/// In that case the function returns `Err(Stopped)`.
pub async fn wait_stop_or_button(
    sys: &impl AccessButtonSignal,
    stop_signal: &impl ReceiveBool,
) -> Result<(), Stopped> {
    while !stop_signal.get() {
        yield_now().await;
        if sys.is_button_signal() {
            return Ok(());
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
pub async fn wait_stop_or_button_or_timeout(
    sys: &(impl AccessTiming + AccessButtonSignal),
    stop_signal: &impl ReceiveBool,
    timeout: DurationMillis,
) -> Result<bool, Stopped> {
    let start_time = sys.get_instant();
    let mut time_left = timeout;
    while !stop_signal.get() {
        sleep_at_most(sys, time_left).await;
        if sys.is_button_signal() {
            return Ok(true);
        }
        let time_passed = sys.get_instant() - start_time;
        time_left = timeout - time_passed;
        if time_left <= 0 {
            return Ok(false);
        }
    }
    Err(Stopped)
}
