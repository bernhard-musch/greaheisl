//! A desktop PC command line application 
//! we can use to test and debug the user interface
//! provided by `greaheisl_lib`.
//!
//! * The LED matrix is visualized as ASCII graphics.
//! * The arrow keys serve as buttons.
//! 
//! Unfortunately, standard terminals do not provide raw keyboard events. 
//! Therefore, this program needs to be run in a terminal that supports the 
//! [kitty keyboard protocol](https://sw.kovidgoyal.net/kitty/keyboard-protocol/).

//use std::thread;
use crossterm::terminal;
use crossterm::{ExecutableCommand, QueueableCommand};
use std::time::{Duration, Instant};

use crossterm::event::{
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use greaheisl_button_processor::AccessButtonSignal;
use greaheisl_lib::system::{RtcTime, SignalFlags, NUM_RELAYS};

use anyhow::Result;
use bitvec::view::BitView;
use chrono::Timelike;
use num::Integer;
use std::io::{stdout, Write};
use std::sync::{Arc, Mutex};

// #[macro_use]
// extern crate greaheisl;

use ambassador::Delegate;
use greaheisl_async::{ambassador_impl_AccessExecutorSignals, ambassador_impl_AccessTiming};
use greaheisl_async::{AccessExecutorSignals, AccessTiming, DurationWrapper};
use greaheisl_async::{DurationMillis, InstantMillis};
use greaheisl_lib::system::buttons::ButtonFlags;

// use greaheisl::show_clock;

/* This was for testing the C static library
#[link(name = "greaheisl")]
extern "C" {
    fn show_clock(imat: *mut u32, hours: u8, minutes: u8);
}
*/

fn write_matrix<F: std::fmt::Write>(writer: &mut F, matrix: &[u32]) -> Result<()> {
    let width = 12;
    let height = 8;
    writer.write_str("+------------+\n\r")?;
    let matrix = matrix.view_bits::<bitvec::order::Msb0>();
    for i_line in 0..height / 2 {
        writer.write_str("|")?;
        let p1 = (i_line * 2) * width;
        let p2 = p1 + width;
        let bits1 = matrix[p1 as usize..(p1 + width) as usize].iter();
        let bits2 = matrix[p2 as usize..(p2 + width) as usize].iter();
        for (bit1, bit2) in bits1.zip(bits2) {
            if *bit1 {
                if *bit2 {
                    writer.write_char('█')?;
                } else {
                    writer.write_char('▀')?;
                }
            } else {
                if *bit2 {
                    writer.write_char('▄')?;
                } else {
                    writer.write_char(' ')?;
                }
            }
        }
        writer.write_str("|")?;
        writer.write_char('\n')?;
        writer.write_char('\r')?;
    }
    writer.write_str("+------------+\n\r")?;
    Ok(())
}

//#[derive(Clone)]
#[derive(Delegate)]
#[delegate(AccessTiming, target = "scheduler")]
#[delegate(AccessExecutorSignals<SignalFlags>,target = "scheduler")]

struct CliCallbacks<S> {
    buttons: Arc<Mutex<ButtonFlags>>,
    scheduler: S,
    relay_states: Arc<Mutex<[bool; greaheisl_lib::system::NUM_RELAYS]>>,
    time_shift_seconds: Mutex<i32>,
}

fn daytime_to_seconds(hour: u8, minute: u8, second: u8) -> i32 {
    (hour as i32) * 3600 + (minute as i32) * 60 + (second as i32)
}

fn seconds_to_daytime(seconds: i32) -> (u8, u8, u8) {
    let (minute, second) = seconds.div_rem(&60);
    let (hour, minute) = minute.div_rem(&60);
    let hour = hour % 24;
    (hour as u8, minute as u8, second as u8)
}

impl<S> greaheisl_lib::system::AccessRtc for CliCallbacks<S> {
    fn get_rtc(&self) -> RtcTime {
        let time_real = chrono::Local::now().time();
        let seconds_real = daytime_to_seconds(
            time_real.hour() as u8,
            time_real.minute() as u8,
            time_real.second() as u8,
        );
        let (hour, minute, second) =
            seconds_to_daytime(seconds_real + *self.time_shift_seconds.lock().unwrap());
        RtcTime {
            hour,
            minute,
            second,
        }
    }
    fn set_rtc(&self, time: &RtcTime) {
        let time_real = chrono::Local::now().time();
        let seconds_real = daytime_to_seconds(
            time_real.hour() as u8,
            time_real.minute() as u8,
            time_real.second() as u8,
        );
        let seconds_set = daytime_to_seconds(time.hour, time.minute, time.second);
        *self.time_shift_seconds.lock().unwrap() = seconds_set - seconds_real;
    }
}

impl<S> greaheisl_lib::system::AccessLedMatrix for CliCallbacks<S> {
    fn set_led_matrix(&self, matrix: &[u32; 3]) {
        let mut s = String::new();
        write_matrix(&mut s, matrix).unwrap();
        stdout().queue(crossterm::cursor::MoveTo(0, 2)).unwrap();
        write!(stdout(), "{}", s).unwrap();
        stdout().flush().unwrap();
    }
}

impl<S> greaheisl_button_processor::AccessButtonState for CliCallbacks<S> {
    type ButtonFlags = ButtonFlags;
    fn get_button_flags(&self) -> ButtonFlags {
        *self.buttons.lock().unwrap()
    }
}

impl<S: AccessExecutorSignals<SignalFlags>> AccessButtonSignal for CliCallbacks<S> {
    fn is_button_signal(&self) -> bool {
        self.get_executor_signals().contains(SignalFlags::Button)
    }
}

impl<S> greaheisl_lib::system::AccessOutputStates for CliCallbacks<S> {
    fn set_relay_states(&self, relais_states: &[bool; greaheisl_lib::system::NUM_RELAYS]) {
        *self.relay_states.lock().unwrap() = *relais_states;
    }
}

fn instantmillis_from_duration(duration: std::time::Duration) -> InstantMillis {
    InstantMillis::from_absolute(duration.as_millis() as u32)
}

fn map_key_codes(key: crossterm::event::KeyCode) -> Option<ButtonFlags> {
    use crossterm::event::KeyCode;
    match key {
        KeyCode::Up => Some(ButtonFlags::Prev),
        KeyCode::Down => Some(ButtonFlags::Next),
        KeyCode::Left => Some(ButtonFlags::Escape),
        KeyCode::Right => Some(ButtonFlags::Enter),
        _ => None,
    }
}

fn run() -> Result<()> {
    let start_instant = Instant::now();
    // let callbacks = CliCallbacks{ event: Arc::new(Mutex::new(ButtonEvent::None)) };
    let instant = instantmillis_from_duration(start_instant.elapsed());
    let executor = greaheisl_async::MiniExecutor::new(instant);
    let buttons = Arc::new(Mutex::new(ButtonFlags::none()));
    let relay_states = Arc::new(Mutex::new([false; NUM_RELAYS]));
    let callbacks = CliCallbacks {
        buttons: buttons.clone(),
        scheduler: executor.scheduler().clone(),
        relay_states: relay_states.clone(),
        time_shift_seconds: Mutex::new(0),
    };
    let task = greaheisl_lib::run(callbacks);
    let mut executor = executor.build(task);
    let mut next_delay_millis: DurationMillis = 100;
    stdout().execute(terminal::Clear(terminal::ClearType::All))?;
    stdout().queue(crossterm::cursor::MoveTo(0, 0)).unwrap();
    write!(stdout(), "GREAHEISL BOARD EMULATOR\n\r").unwrap();
    stdout().queue(crossterm::cursor::MoveTo(0, 8)).unwrap();
    write!(
        stdout(),
        "Use a terminal supporting the kitty keyboard protocol.\n\r"
    )
    .unwrap();
    write!(
        stdout(),
        "Use arrow keys as buttons. Press Ctrl-C to exit.\n\r"
    )
    .unwrap();
    //stdout().execute(crossterm::cursor::SavePosition)?;
    loop {
        let mut signals = SignalFlags::none();
        use crossterm::event as cev;
        use crossterm::event::Event as Cev;
        use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
        if cev::poll(Duration::from_millis(next_delay_millis as u64))? {
            match cev::read()? {
                Cev::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press | KeyEventKind::Repeat,
                    state: _,
                }) => {
                    break;
                }
                Cev::Key(KeyEvent {
                    code,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: _,
                }) => {
                    if let Some(pressed_button) = map_key_codes(code) {
                        let mut buttons = buttons.lock().unwrap();
                        *buttons = *buttons | pressed_button;
                        signals = signals | SignalFlags::Button;
                    }
                }
                Cev::Key(KeyEvent {
                    code,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Release,
                    state: _,
                }) => {
                    if let Some(released_button) = map_key_codes(code) {
                        let mut buttons = buttons.lock().unwrap();
                        *buttons = *buttons & (!released_button);
                        signals = signals | SignalFlags::Button;
                    }
                }
                _ => {}
            }
        }
        stdout().queue(crossterm::cursor::MoveTo(0, 1)).unwrap();
        write!(
            stdout(),
            "Button states: {:04b}",
            buttons.lock().unwrap().bits()
        )
        .unwrap();
        let relay_bits: String = relay_states
            .lock()
            .unwrap()
            .iter()
            .map(|b| if *b { '1' } else { '0' })
            .collect();
        write!(stdout(), "  Relay states: {}", relay_bits).unwrap();
        stdout().flush().unwrap();
        /* This was for testing the original C library interface
        let mut screen : [u32;3] = [0;3];
        let time = chrono::Local::now().time();
        unsafe{ show_clock(screen.as_mut_ptr(), time.minute() as u8, time.second() as u8); }

        let mut s = String::new();
        write_matrix(&mut s,&screen)?;
        //stdout().execute(crossterm::cursor::RestorePosition)?;
        stdout()
            .queue(crossterm::cursor::MoveTo(0,0))?;
        write!(stdout(),"{}",s)?;
        stdout().flush()?;
        */
        let instant = instantmillis_from_duration(start_instant.elapsed());
        let Some(delay_request) = executor.step(instant, signals) else {
            break;
        };
        next_delay_millis = i32::min(delay_request, 2000);
    }
    Ok(())
}

fn main() -> Result<()> {
    terminal::enable_raw_mode()?;
    stdout().execute(PushKeyboardEnhancementFlags(
        KeyboardEnhancementFlags::REPORT_EVENT_TYPES,
    ))?;
    let res = run();
    stdout().queue(crossterm::cursor::MoveTo(0, 11))?;
    stdout().execute(PopKeyboardEnhancementFlags)?;
    terminal::disable_raw_mode()?;
    res
}
