use crate::system::buttons::{ButtonFlags, SysButtonProcessor};
use crate::system::AccessLedMatrix;
use crate::ScheduledOutEntry;
use greaheisl_async::AccessTiming;
use greaheisl_bitvecimg::font::fitzl_font::FitzlFontNarrowNum;
use greaheisl_bitvecimg::font::typeset::{TextLinePrinter, TextPrinterTrait};
use greaheisl_bitvecimg::ImageRegionMut;
use greaheisl_button_processor::ButtonEvent;

use crate::ui::components::{
    choose_duration, choose_time, selection, DisplayImage, SelectionResponse, SelectionState,
};

use enum_iterator::{next_cycle, previous_cycle, Sequence};

#[derive(Default, Clone, Copy, Sequence)]
enum MenuState {
    #[default]
    Duration,
    StartTime,
}

impl SelectionState for MenuState {
    type SelectionItem = Self;

    fn item(&self) -> &Self::SelectionItem {
        self
    }

    fn next(&mut self) -> bool {
        let Some(new_value) = next_cycle(self) else {
            return false;
        };
        *self = new_value;
        true
    }

    fn previous(&mut self) -> bool {
        let Some(new_value) = previous_cycle(self) else {
            return false;
        };
        *self = new_value;
        true
    }
}

impl DisplayImage for MenuState {
    fn display_image(&self, canvas: ImageRegionMut) {
        let mut printer = TextLinePrinter::new(canvas, FitzlFontNarrowNum {});
        match self {
            MenuState::Duration => {
                printer.print_str("DAU").unwrap();
            }
            MenuState::StartTime => {
                printer.print_str("STA").unwrap();
            }
        }
    }
}

pub async fn menu_scheduled_entry(
    sys: &(impl AccessLedMatrix + AccessTiming),
    btns: &SysButtonProcessor,
    settings: &mut ScheduledOutEntry,
) -> bool {
    let mut current_item = MenuState::default();
    loop {
        selection(sys, btns, &mut current_item).await;
        match btns.event() {
            ButtonEvent::None => break true, // timeout => exit all menus
            ButtonEvent::Press(ButtonFlags::Escape) => break false, // user wants to get back
            ButtonEvent::Press(ButtonFlags::Enter) => match current_item {
                MenuState::Duration => {
                    if choose_duration(sys, btns, &mut settings.duration).await {
                        break true;
                    }
                }
                MenuState::StartTime => {
                    let response = choose_time(
                        sys,
                        btns,
                        &mut settings.start_hour,
                        &mut settings.start_minute,
                    )
                    .await;
                    let SelectionResponse::Back = response else {
                        break true;
                    };
                }
            },
            _ => {}
        }
    }
}
