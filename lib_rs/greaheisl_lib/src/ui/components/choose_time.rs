//use crate::ui::bitvecimg_printer::{BitVecImgPrinterTrait,BlinkingBitVecImgPrinter};
use greaheisl_bitvecimg::font::typeset::{
    canvas::CarbonCopyCanvas, TextLinePrinter, TextPrinterTrait,
};
use greaheisl_bitvecimg::Image;

use crate::system::buttons::{wait_button_press2_or_timeout, ButtonFlags, SysButtonProcessor};
use crate::system::AccessLedMatrix;
use crate::ui::display::run_blinking_led_matrix;
use greaheisl_async::{AccessTiming, Timer};
use greaheisl_bitvecimg::font::fitzl_font::FitzlFontNarrowNum;
use greaheisl_button_processor::ButtonEvent;

use super::{SelectionResponse, BLINK_DELAY_CHANGE_VALUE, BLINK_DELAY_CONFIRM_VALUE, MENU_TIMEOUT};

enum Phase {
    Hour,
    Minute,
}

pub async fn choose_time(
    sys: &(impl AccessLedMatrix + AccessTiming),
    btns: &SysButtonProcessor,
    hour: &mut u8,
    minute: &mut u8,
) -> SelectionResponse {
    let mut hour_state = *hour;
    let mut minute_state = *minute;
    let mut phase = Phase::Hour;
    loop {
        match phase {
            Phase::Hour => {
                let response = choose_hour(sys, btns, &mut hour_state, minute_state).await;
                match response {
                    SelectionResponse::Ok => phase = Phase::Minute,
                    SelectionResponse::Back | SelectionResponse::Timeout => return response,
                }
            }
            Phase::Minute => {
                let response = choose_minute(sys, btns, hour_state, &mut minute_state).await;
                match response {
                    SelectionResponse::Ok => break,
                    SelectionResponse::Back => phase = Phase::Hour,
                    SelectionResponse::Timeout => return response,
                }
            }
        }
    }
    // confirm that a new time has been chosen by blinking quickly
    let sheets = with_blinking_printer(&[true, true, true, true], |printer| {
        printer.print_uint::<_, 2>(hour_state).unwrap();
        printer.skip(1);
        printer.print_uint::<_, 2>(minute_state).unwrap();
    });
    run_blinking_led_matrix(sys, &sheets, BLINK_DELAY_CONFIRM_VALUE, async {
        Timer::new(sys, BLINK_DELAY_CONFIRM_VALUE * 5).wait().await;
    })
    .await;
    *hour = hour_state;
    *minute = minute_state;
    SelectionResponse::Ok
}

async fn choose_hour(
    sys: &(impl AccessLedMatrix + AccessTiming),
    btns: &SysButtonProcessor,
    hour: &mut u8,
    minute: u8,
) -> SelectionResponse {
    loop {
        let sheets = with_blinking_printer(&[true, true, false, false], |printer| {
            printer.print_uint::<_, 2>(*hour).unwrap();
            printer.skip(1);
            printer.print_uint::<_, 2>(minute).unwrap();
        });
        run_blinking_led_matrix(
            sys,
            &sheets,
            BLINK_DELAY_CHANGE_VALUE,
            wait_button_press2_or_timeout(sys, btns, MENU_TIMEOUT),
        )
        .await;
        match btns.event() {
            ButtonEvent::None => break SelectionResponse::Timeout, // timeout => exit all menus
            ButtonEvent::Press(ButtonFlags::Escape) => break SelectionResponse::Back, // user wants to get back
            ButtonEvent::Press(ButtonFlags::Prev) | ButtonEvent::Repeat(ButtonFlags::Prev) => {
                if *hour > 0 {
                    *hour -= 1;
                } else {
                    *hour = 23;
                }
            }
            ButtonEvent::Press(ButtonFlags::Next) | ButtonEvent::Repeat(ButtonFlags::Next) => {
                if *hour < 23 {
                    *hour += 1;
                } else {
                    *hour = 0;
                }
            }
            ButtonEvent::Press(ButtonFlags::Enter) => {
                break SelectionResponse::Ok; // successful setting, go on with minutes
            }
            _ => {}
        }
    }
}

async fn choose_minute(
    sys: &(impl AccessLedMatrix + AccessTiming),
    btns: &SysButtonProcessor,
    hour: u8,
    minute: &mut u8,
) -> SelectionResponse {
    loop {
        let sheets = with_blinking_printer(&[false, false, true, true], |printer| {
            printer.print_uint::<_, 2>(hour).unwrap();
            printer.skip(1);
            printer.print_uint::<_, 2>(*minute).unwrap();
        });
        run_blinking_led_matrix(
            sys,
            &sheets,
            BLINK_DELAY_CHANGE_VALUE,
            wait_button_press2_or_timeout(sys, btns, MENU_TIMEOUT),
        )
        .await;
        match btns.event() {
            ButtonEvent::None => break SelectionResponse::Timeout, // timeout => exit all menus
            ButtonEvent::Press(ButtonFlags::Escape) => break SelectionResponse::Back, // user wants to get back
            ButtonEvent::Press(ButtonFlags::Prev) => {
                if *minute > 0 {
                    *minute -= 1;
                } else {
                    *minute = 59;
                }
            }
            ButtonEvent::Repeat(ButtonFlags::Prev) => {
                if *minute > 0 {
                    *minute -= 1;
                    *minute = (*minute / 10) * 10; // round down to next multiple of 10
                } else {
                    *minute = 50;
                }
            }
            ButtonEvent::Press(ButtonFlags::Next) => {
                if *minute < 59 {
                    *minute += 1;
                } else {
                    *minute = 0;
                }
            }
            ButtonEvent::Repeat(ButtonFlags::Next) => {
                if *minute < 50 {
                    *minute += 10;
                    *minute = (*minute / 10) * 10; // round down to next multiple of 10
                } else {
                    *minute = 0;
                }
            }
            ButtonEvent::Press(ButtonFlags::Enter) => {
                break SelectionResponse::Ok; // successful setting, go on with minutes
            }
            _ => {}
        }
    }
}

fn with_blinking_printer<'a>(
    mask: &'a [bool],
    fcn: impl FnOnce(
        &mut TextLinePrinter<CarbonCopyCanvas<'a, &mut Image<12, 8, 3>>, FitzlFontNarrowNum>,
    ),
) -> [Image<12, 8, 3>; 2] {
    let mut sheets = [Image::zero(), Image::zero()];
    let [ref mut front_sheet, ref mut back_sheet] = sheets;
    let blink_canvas = CarbonCopyCanvas::new(mask, front_sheet, back_sheet);
    let mut printer = TextLinePrinter::new(blink_canvas, FitzlFontNarrowNum {});
    fcn(&mut printer);
    sheets
}
