use crate::system::buttons::{ButtonFlags, SysButtonProcessor};
use crate::system::{AccessLedMatrix, NUM_RELAYS};
use crate::{Settings, MAX_SCHEDULED_ENTRIES};
use greaheisl_async::AccessTiming;
use greaheisl_bitvecimg::font::fitzl_font::FitzlFontNarrowNum;
use greaheisl_bitvecimg::font::typeset::{TextLinePrinter, TextPrinterTrait};
use greaheisl_bitvecimg::ImageRegionMut;
use greaheisl_button_processor::ButtonEvent;

use super::immediate_out::menu_immediate_out;
use super::scheduled_entry::menu_scheduled_entry;

use crate::ui::components::{selection, DisplayImage, SelectionState};

#[derive(Debug, PartialEq)]
enum MainMenuItem {
    ImmediateOut { channel: usize },
    ScheduledOut { channel: usize, i_entry: usize },
}

impl DisplayImage for MainMenuItem {
    fn display_image(&self, canvas: ImageRegionMut) {
        let mut printer = TextLinePrinter::new(canvas, FitzlFontNarrowNum {});
        match self {
            MainMenuItem::ImmediateOut { channel } => {
                printer.print_str("J").unwrap();
                printer.print_uint::<_, 1>(channel + 1).unwrap();
            }
            MainMenuItem::ScheduledOut { channel, i_entry } => {
                printer.print_str("S").unwrap();
                printer.print_uint::<_, 1>(channel + 1).unwrap();
                printer.skip(2);
                printer.print_uint::<_, 1>(i_entry + 1).unwrap();
            }
        }
    }
}

impl SelectionState for MainMenuItem {
    type SelectionItem = MainMenuItem;

    fn item(&self) -> &Self::SelectionItem {
        return self;
    }

    fn next(&mut self) -> bool {
        match *self {
            MainMenuItem::ImmediateOut { channel } => {
                if channel < NUM_RELAYS - 1 {
                    *self = MainMenuItem::ImmediateOut {
                        channel: channel + 1,
                    };
                    true
                } else {
                    *self = MainMenuItem::ScheduledOut {
                        channel: 0,
                        i_entry: 0,
                    };
                    true
                }
            }
            MainMenuItem::ScheduledOut { channel, i_entry } => {
                if i_entry < MAX_SCHEDULED_ENTRIES - 1 {
                    *self = MainMenuItem::ScheduledOut {
                        channel,
                        i_entry: i_entry + 1,
                    };
                    true
                } else {
                    if channel < NUM_RELAYS - 1 {
                        *self = MainMenuItem::ScheduledOut {
                            channel: channel + 1,
                            i_entry: 0,
                        };
                        true
                    } else {
                        *self = MainMenuItem::ImmediateOut { channel: 0 };
                        true
                    }
                }
            }
        }
    }

    fn previous(&mut self) -> bool {
        match *self {
            MainMenuItem::ImmediateOut { channel } => {
                if channel != 0 {
                    *self = MainMenuItem::ImmediateOut {
                        channel: channel - 1,
                    };
                    true
                } else {
                    let channel = NUM_RELAYS - 1;
                    *self = MainMenuItem::ScheduledOut {
                        channel,
                        i_entry: MAX_SCHEDULED_ENTRIES - 1,
                    };
                    true
                }
            }
            MainMenuItem::ScheduledOut { channel, i_entry } => {
                if channel != 0 {
                    if i_entry != 0 {
                        *self = MainMenuItem::ScheduledOut {
                            channel,
                            i_entry: i_entry - 1,
                        };
                        true
                    } else {
                        let channel = channel - 1;
                        *self = MainMenuItem::ScheduledOut {
                            channel,
                            i_entry: MAX_SCHEDULED_ENTRIES - 1,
                        };
                        true
                    }
                } else {
                    let channel = NUM_RELAYS - 1;
                    *self = MainMenuItem::ImmediateOut { channel };
                    true
                }
            }
        }
    }
}

pub async fn menu_main(
    sys: &(impl AccessLedMatrix + AccessTiming),
    btns: &SysButtonProcessor,
    settings: &mut Settings,
) {
    let mut current_item = MainMenuItem::ImmediateOut { channel: 0 };
    loop {
        selection(sys, btns, &mut current_item).await;
        match btns.event() {
            ButtonEvent::Press(ButtonFlags::Escape) => {
                break;
            } // user wants to leave
            ButtonEvent::Press(ButtonFlags::Enter) => match current_item {
                MainMenuItem::ImmediateOut { channel } => {
                    if menu_immediate_out(sys, btns, &mut settings.immediate_out[channel]).await {
                        break;
                    }
                }
                MainMenuItem::ScheduledOut { channel, i_entry } => {
                    if menu_scheduled_entry(
                        sys,
                        btns,
                        &mut settings.scheduled_out[channel][i_entry],
                    )
                    .await
                    {
                        break;
                    }
                }
            },
            _ => {} //? should not occur
        }
    }
}

/* obsolete
fn get_index_last_schedulable_entry(entries: &[Option<ScheduledOutEntry>;MAX_SCHEDULED_ENTRIES]) -> usize {
    let mut count = 0;
    for k in entries {
        if k.is_none() { break }
        count += 1;
    }
    usize::min(count,MAX_SCHEDULED_ENTRIES)
}

fn is_index_last_schedulable_entry(entries: &[Option<ScheduledOutEntry>;MAX_SCHEDULED_ENTRIES], i: usize) -> bool {
    i == MAX_SCHEDULED_ENTRIES || entries[i].is_none()
}
*/
