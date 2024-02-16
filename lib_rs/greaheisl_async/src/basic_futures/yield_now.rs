use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

/// Cooperatively gives up a timeslice to the task scheduler.
///
/// In other words, stops execution here and passes control back to the caller
/// until `poll()` is invoked again.
///
/// # Note:
///
/// The source code for `yield_now()` has been copied from
/// [the async version of the Rust standard library](https://github.com/async-rs/async-std)

#[inline]
pub async fn yield_now() {
    YieldNow(false).await
}

struct YieldNow(bool);

impl Future for YieldNow {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.0 {
            self.0 = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}
