use greaheisl_async::DurationMillis;
use greaheisl_bitvecimg::ImageRegionMut;

pub const MENU_TIMEOUT: DurationMillis = 10000;
pub const BLINK_DELAY_CHANGE_VALUE: DurationMillis = 400;
pub const BLINK_DELAY_CONFIRM_VALUE: DurationMillis = 200;

#[derive(Clone, Copy)]
pub enum SelectionResponse {
    Ok,
    Back,
    Timeout,
}

mod choose_time;
mod duration;
mod selection;

pub use choose_time::choose_time;
pub use duration::{choose_duration, print_duration};
pub use selection::{selection, SelectionState};

pub trait DisplayImage {
    fn display_image(&self, canvas: ImageRegionMut);
}
