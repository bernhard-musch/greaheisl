use crate::system::buttons::{ButtonFlags, SysButtonProcessor};
use crate::system::{AccessLedMatrix, AccessRtc};
use core::cell::Cell;
use enum_iterator::{next_cycle, previous_cycle, Sequence};
use greaheisl_button_processor::{wait_button_press_or_timeout, ButtonEvent};
//use bitmask_enum::bitmask;
use greaheisl_async::join2;
use greaheisl_async::{AccessTiming, Timer};

use super::display::show_clock;

async fn idly_show_clock(
    sys: &(impl AccessLedMatrix + AccessTiming + AccessLedMatrix + AccessRtc),
    btns: &SysButtonProcessor,
    longer: bool,
) {
    let timeout = match longer {
        true => 5000,
        false => 20000,
    };
    let stop_signal = Cell::new(false);
    join2(
        async {
            wait_button_press_or_timeout(sys, btns, timeout).await;
            stop_signal.set(true);
        },
        async {
            while !stop_signal.get() {
                show_clock(sys);
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
}

async fn idly_show_flower(
    sys: &(impl AccessLedMatrix + AccessTiming + AccessLedMatrix),
    btns: &SysButtonProcessor,
    longer: bool,
) {
    use bitvec::prelude::*;
    use greaheisl_bitvecimg::Image;
    const FLOWER: Image<12, 8, 3> = Image(bitarr![const u32,Msb0;
        0,0,0,1,1,0,0,1,1,0,0,0,
        0,0,1,0,0,1,1,0,0,1,0,0,
        0,0,1,0,0,1,1,0,0,1,0,0,
        0,0,0,1,1,0,0,1,1,0,0,0,
        0,0,0,1,1,0,0,1,1,0,0,0,
        0,0,1,0,0,1,1,0,0,1,0,0,
        0,0,1,0,0,1,1,0,0,1,0,0,
        0,0,0,1,1,0,0,1,1,0,0,0,
    ]);

    let timeout = match longer {
        true => 1000,
        false => 5000,
    };
    sys.set_led_matrix(&FLOWER.0.into_inner());
    wait_button_press_or_timeout(sys, btns, timeout).await;
}

#[derive(Clone, Copy, Debug, PartialEq, Sequence)]
pub enum IdleScenes {
    Clock,
    Flower,
}

pub struct IdleDisplay {
    idle_scene: IdleScenes,
}

impl IdleDisplay {
    pub fn new() -> Self {
        Self {
            idle_scene: IdleScenes::first().unwrap(),
        }
    }

    pub fn idle_scene(&self) -> IdleScenes {
        self.idle_scene
    }

    pub async fn run(
        &mut self,
        sys: &(impl AccessLedMatrix + AccessTiming + AccessRtc),
        btns: &SysButtonProcessor,
    ) {
        let mut longer = false;
        loop {
            match self.idle_scene {
                IdleScenes::Clock => idly_show_clock(sys, &btns, longer).await,
                IdleScenes::Flower => idly_show_flower(sys, &btns, longer).await,
            }
            match btns.event() {
                ButtonEvent::Press(ButtonFlags::Prev) | ButtonEvent::Repeat(ButtonFlags::Prev) => {
                    self.idle_scene = previous_cycle(&self.idle_scene).unwrap();
                    longer = true;
                }
                ButtonEvent::Press(ButtonFlags::Next) | ButtonEvent::Repeat(ButtonFlags::Next) => {
                    self.idle_scene = next_cycle(&self.idle_scene).unwrap();
                    longer = true;
                }
                ButtonEvent::None => {
                    self.idle_scene = next_cycle(&self.idle_scene).unwrap();
                    longer = false;
                }
                ButtonEvent::Press(ButtonFlags::Enter) => {
                    break;
                }
                _ => {}
            }
        }
    }
}
