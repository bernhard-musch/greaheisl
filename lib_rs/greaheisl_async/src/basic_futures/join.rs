use core::future::Future;
use core::pin::{pin, Pin};
use core::task::{Context, Poll};
use pin_project::pin_project;

/// runs two futures "in parallel"
///
/// More precisely, each time `poll()` is invoked for this future,
/// `poll()` of `future1` and `future2` are invoked in sequence,
/// until results have been collected from both futures.
/// The return value is a tuple with the return values from `future1` und `future2`.
pub async fn join2<F1: Future<Output = T>, F2: Future<Output = U>, T, U>(
    future1: F1,
    future2: F2,
) -> (T, U) {
    let future1 = pin!(future1);
    let future2 = pin!(future2);
    let pair = Join2 {
        future1,
        future2,
        retval1: None,
        retval2: None,
    };
    pair.await
}

#[pin_project]
struct Join2<F1, F2, T, U> {
    #[pin]
    future1: F1,
    #[pin]
    future2: F2,
    retval1: Option<T>,
    retval2: Option<U>,
}

impl<'a, F1: Future<Output = T>, F2: Future<Output = U>, T, U> Future for Join2<F1, F2, T, U> {
    type Output = (T, U);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if this.retval1.is_none() {
            let res = this.future1.poll(cx);
            if let Poll::Ready(retval) = res {
                *this.retval1 = Some(retval);
            }
        }
        if this.retval2.is_none() {
            let res = this.future2.poll(cx);
            if let Poll::Ready(retval) = res {
                *this.retval2 = Some(retval);
            }
        }
        if this.retval1.is_some() && this.retval2.is_some() {
            let retval1 = this.retval1.take().unwrap();
            let retval2 = this.retval2.take().unwrap();
            Poll::Ready((retval1, retval2))
        } else {
            Poll::Pending
        }
    }
}
