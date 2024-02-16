use crate::system::{AccessLedMatrix, AccessRtc};
use bitvec::{order::Msb0, view::BitView};
use core::cell::Cell;
use core::future::Future;
use greaheisl_async::join2;
use greaheisl_async::DurationMillis;
use greaheisl_async::{AccessTiming, Timer};
use greaheisl_bitvecimg::font::fitzl_font::FitzlFontNarrowNum;
use greaheisl_bitvecimg::font::typeset::{TextLinePrinter, TextPrinterTrait};
use greaheisl_bitvecimg::{BitVecImgViewMut, Image};

/* obsolete; formerly used in demo function `interface_c::show_clock()`*/
/*
pub fn display_clock(imat: &mut impl ImageMatrixViewMut, hours: u8, minutes: u8) {
    let mut printer = ImageMatrixPrinter::new(imat);
    printer.print_uint::<_,2>(hours);
    printer.print_space(1);
    printer.print_uint::<_,2>(minutes);
}
*/

pub fn with_led_printer(
    sys: &impl AccessLedMatrix,
    f: impl FnOnce(&mut TextLinePrinter<&mut Image<12, 8, 3>, &FitzlFontNarrowNum>),
) {
    let mut imat = Image::<12, 8, 3>::zero();
    let mut printer = TextLinePrinter::new(&mut imat, &FitzlFontNarrowNum {});
    f(&mut printer);
    sys.set_led_matrix(&imat.0.into_inner());
}

pub async fn run_blinking_led_matrix<T>(
    sys: &(impl AccessTiming + AccessLedMatrix),
    matrices: &[Image<12, 8, 3>; 2],
    blink_delay: DurationMillis,
    fut: impl Future<Output = T>,
) -> T {
    let stop_signal = Cell::new(false);
    join2(
        async {
            let res = fut.await;
            stop_signal.set(true);
            res
        },
        async {
            let mut state = 0usize;
            while !stop_signal.get() {
                sys.set_led_matrix(&matrices[state].0.into_inner());
                let timer = Timer::new(sys, blink_delay);
                while timer.yield_if_time_left().await && !stop_signal.get() {}
                state ^= 1;
            }
        },
    )
    .await
    .0
}

pub fn show_clock(sys: &(impl AccessLedMatrix + AccessRtc)) {
    let rtc_time = sys.get_rtc();
    let mut imat = Image::<12, 8, 3>::zero();
    {
        let mut printer = TextLinePrinter::new(&mut imat, &FitzlFontNarrowNum {});
        printer.print_uint::<_, 2>(rtc_time.hour).unwrap();
        printer.skip(1);
        printer.print_uint::<_, 2>(rtc_time.minute).unwrap();
    }
    let second_bits = rtc_time.second as u32;
    let imat_bits = imat.row_bits_mut(7); // access to the bits at y=7
    imat_bits[0..6].copy_from_bitslice(&second_bits.view_bits::<Msb0>()[26..32]);
    sys.set_led_matrix(&imat.0.into_inner());
}
