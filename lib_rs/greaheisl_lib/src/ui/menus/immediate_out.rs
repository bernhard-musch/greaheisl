use core::cell::Cell;

use crate::system::buttons::{ButtonFlags, SysButtonProcessor};
use crate::system::AccessLedMatrix;
use crate::ui::components::{choose_duration, print_duration, MENU_TIMEOUT};
use crate::ui::display::with_led_printer;
use crate::ImmediateOutEntry;
use greaheisl_async::join2;
use greaheisl_async::{AccessTiming, Timer};
use greaheisl_async::{DurationMillis, InstantMillis};
use greaheisl_bitvecimg::font::typeset::TextPrinterTrait;
use greaheisl_bitvecimg::{BitVecImgViewMut, Image};
use greaheisl_button_processor::{wait_button_press_or_timeout, ButtonEvent};

pub async fn menu_immediate_out(
    sys: &(impl AccessLedMatrix + AccessTiming),
    btns: &SysButtonProcessor,
    settings: &mut Option<ImmediateOutEntry>,
) -> bool {
    loop {
        let stop_signal = Cell::new(false);
        join2(
            async {
                wait_button_press_or_timeout(sys, btns, MENU_TIMEOUT).await;
                stop_signal.set(true);
            },
            async {
                while !stop_signal.get() {
                    let time_left = get_time_left(settings, sys.get_instant());
                    if time_left > 0 {
                        let mut matrix = Image::<12, 8, 3>::zero();
                        print_duration(matrix.as_region_mut(), time_left);
                        sys.set_led_matrix(&matrix.0.into_inner());
                    } else {
                        with_led_printer(sys, |printer| printer.print_str("AUS").unwrap());
                    }
                    let update_timer = Timer::new(sys, 1000);
                    while update_timer.yield_if_time_left().await {
                        if stop_signal.get() {
                            break;
                        }
                    }
                }
            },
        )
        .await;
        match btns.event() {
            ButtonEvent::None => break true, // timeout => exit all menus
            ButtonEvent::Press(ButtonFlags::Escape) => break false, // user wants to get back
            ButtonEvent::Press(ButtonFlags::Enter) => {
                let mut duration = get_time_left(settings, sys.get_instant());
                let is_set = choose_duration(sys, btns, &mut duration).await;
                if duration == 0 {
                    *settings = None;
                } else {
                    *settings = Some(ImmediateOutEntry {
                        start: sys.get_instant(),
                        duration,
                    });
                }
                break is_set;
            }
            _ => {}
        }
    }
}

fn get_time_left(entry: &Option<ImmediateOutEntry>, now: InstantMillis) -> DurationMillis {
    let Some(ImmediateOutEntry { start, duration }) = entry else {
        return 0;
    };
    let time_left = *duration - (now - *start);
    DurationMillis::max(0, time_left)
}
