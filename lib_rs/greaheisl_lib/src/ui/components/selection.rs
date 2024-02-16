use crate::system::buttons::{ButtonFlags, SysButtonProcessor};
use crate::system::AccessLedMatrix;
use greaheisl_async::AccessTiming;
use greaheisl_bitvecimg::{BitVecImgViewMut, Image};
use greaheisl_button_processor::{wait_button_press_or_timeout, ButtonEvent};

use super::{DisplayImage, MENU_TIMEOUT};

pub trait SelectionState {
    type SelectionItem;
    fn item(&self) -> &Self::SelectionItem;
    /// returns false if state does not change (end of selection item list reached);
    /// always true for cyclic lists
    fn next(&mut self) -> bool;
    /// returns false if state does not change (beginning of selection item list reached);
    /// always true for cyclic lists
    fn previous(&mut self) -> bool;
}

pub async fn selection<S: DisplayImage>(
    sys: &(impl AccessLedMatrix + AccessTiming),
    btns: &SysButtonProcessor,
    state: &mut impl SelectionState<SelectionItem = S>,
) {
    loop {
        let mut imat = Image::<12, 8, 3>::zero();
        state.item().display_image(imat.as_region_mut());
        sys.set_led_matrix(&imat.0.into_inner());
        wait_button_press_or_timeout(sys, btns, MENU_TIMEOUT).await;
        match btns.event() {
            ButtonEvent::None => break,                       // timeout => exit
            ButtonEvent::Press(ButtonFlags::Escape) => break, // user wants to leave
            ButtonEvent::Press(ButtonFlags::Prev) => {
                state.previous();
            }
            ButtonEvent::Press(ButtonFlags::Next) => {
                state.next();
            }
            ButtonEvent::Press(ButtonFlags::Enter) => break, // user has accepted
            _ => {}
        }
    }
}
