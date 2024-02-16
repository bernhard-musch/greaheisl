use crate::system::buttons::{ButtonFlags, SysButtonProcessor, BUTTON_HOLD_DURATION};
use crate::system::System;
use crate::ui::components::{choose_time, SelectionResponse};
use core::cell::RefCell;
use greaheisl_button_processor::{
    ButtonEvent, ButtonProcessorOptions, CheckHoldButton, CheckHoldButtonResult,
};

use crate::Settings;

mod components;
mod display;
mod idle_display;
mod menus;

use idle_display::{IdleDisplay, IdleScenes};
use menus::menu_main;

pub async fn run_ui(sys: &impl System, settings: &RefCell<Settings>) {
    // main loop
    let btns = SysButtonProcessor::new(ButtonProcessorOptions::default());
    btns.run(sys, async {
        let mut idle_display = IdleDisplay::new();
        loop {
            idle_display.run(sys, &btns).await;
            match btns.event() {
                ButtonEvent::Press(ButtonFlags::Enter) => {
                    let res = CheckHoldButton::new(sys, &btns, BUTTON_HOLD_DURATION)
                        .wait()
                        .await;
                    match res {
                        CheckHoldButtonResult::ReleaseEarly => {
                            let mut new_settings = settings.borrow().clone();
                            menu_main(sys, &btns, &mut new_settings).await;
                            *settings.borrow_mut() = new_settings;
                        }
                        CheckHoldButtonResult::Hold => {
                            let mut time = sys.get_rtc();
                            let IdleScenes::Clock = idle_display.idle_scene() else {
                                continue;
                            };
                            let SelectionResponse::Ok =
                                choose_time(sys, &btns, &mut time.hour, &mut time.minute).await
                            else {
                                continue;
                            };
                            time.second = 0;
                            sys.set_rtc(&time);
                        }
                        CheckHoldButtonResult::Other => {}
                    }
                }
                _ => {}
            }
        }
    })
    .await;
}
