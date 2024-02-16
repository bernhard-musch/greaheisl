use crate::system::buttons::{ButtonFlags, SysButtonProcessor};
use crate::system::AccessLedMatrix;
use greaheisl_async::{AccessTiming, Timer};
use greaheisl_button_processor::{wait_button_press_or_timeout, ButtonEvent};
//use crate::ui::bitvecimg_printer::{BitVecImgPrinter,BitVecImgPrinterTrait};
use crate::ui::display::run_blinking_led_matrix;
use greaheisl_async::DurationMillis;
use greaheisl_bitvecimg::font::fitzl_font::FitzlFontNarrowNum;
use greaheisl_bitvecimg::font::typeset::{TextLinePrinter, TextPrinterTrait};
use greaheisl_bitvecimg::{BitVecImgViewMut, Image, ImageRegionMut};

use super::{BLINK_DELAY_CHANGE_VALUE, BLINK_DELAY_CONFIRM_VALUE, MENU_TIMEOUT};

pub fn print_duration(matrix: ImageRegionMut, duration: DurationMillis) {
    let mut printer = TextLinePrinter::new(matrix, FitzlFontNarrowNum {});
    let number: DurationMillis;
    let unit_code: &'static str;
    if duration <= 0 {
        number = 0;
        unit_code = "";
    } else if duration < 100 * 1000 {
        number = duration / 1000;
        unit_code = "S";
    } else if duration < 100 * 60 * 1000 {
        number = duration / (60 * 1000);
        unit_code = "M";
    } else if duration < 100 * 60 * 60 * 1000 {
        number = duration / (60 * 60 * 1000);
        unit_code = "H";
    } else {
        number = duration / (24 * 60 * 60 * 1000);
        unit_code = "D";
    }
    printer.print_uint::<_, 2>(number as u8).unwrap();
    printer.print_str(unit_code).unwrap();
}

const SELECTABLE_DURATIONS: [DurationMillis; 15] = [
    0,
    1000 * 30,
    1000 * 60,
    1000 * 60 * 2,
    1000 * 60 * 3,
    1000 * 60 * 5,
    1000 * 60 * 10,
    1000 * 60 * 30,
    1000 * 60 * 60,
    1000 * 60 * 60 * 2,
    1000 * 60 * 60 * 3,
    1000 * 60 * 60 * 6,
    1000 * 60 * 60 * 12,
    1000 * 60 * 60 * 18,
    1000 * 60 * 60 * 24,
];

pub async fn choose_duration(
    sys: &(impl AccessLedMatrix + AccessTiming),
    btns: &SysButtonProcessor,
    setting: &mut DurationMillis,
) -> bool {
    let mut menu_state = *setting;
    loop {
        let mut blink_matrices = [Image::<12, 8, 3>::zero(), Image::<12, 8, 3>::zero()];
        if menu_state > 0 {
            print_duration(blink_matrices[0].as_region_mut(), menu_state);
        } else {
            TextLinePrinter::new(blink_matrices[0].as_region_mut(), FitzlFontNarrowNum {})
                .print_str("AUS")
                .unwrap();
        }
        run_blinking_led_matrix(
            sys,
            &blink_matrices,
            BLINK_DELAY_CHANGE_VALUE,
            wait_button_press_or_timeout(sys, btns, MENU_TIMEOUT),
        )
        .await;
        match btns.event() {
            ButtonEvent::None => break true, // timeout => exit all menus
            ButtonEvent::Press(ButtonFlags::Escape) => break false, // user wants to get back
            ButtonEvent::Press(ButtonFlags::Prev) | ButtonEvent::Repeat(ButtonFlags::Prev) => {
                // find next smaller value in table of selectable durations
                let mut found_dur = 0;
                for sel_dur in SELECTABLE_DURATIONS {
                    if sel_dur < menu_state {
                        found_dur = sel_dur;
                    } else {
                        break;
                    }
                }
                menu_state = found_dur;
            }
            ButtonEvent::Press(ButtonFlags::Next) | ButtonEvent::Repeat(ButtonFlags::Next) => {
                // find next larger value in table of selectable durations
                let mut found_dur = 0;
                for sel_dur in SELECTABLE_DURATIONS {
                    found_dur = sel_dur;
                    if sel_dur > menu_state {
                        break;
                    }
                }
                menu_state = found_dur;
            }
            ButtonEvent::Press(ButtonFlags::Enter) => {
                run_blinking_led_matrix(sys, &blink_matrices, BLINK_DELAY_CONFIRM_VALUE, async {
                    Timer::new(sys, BLINK_DELAY_CONFIRM_VALUE * 5).wait().await;
                })
                .await;
                *setting = menu_state;
                break true; // successful setting
            }
            _ => {}
        }
    }
}
