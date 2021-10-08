use std::{future::Future, task::Poll};

struct Mock {
    n: u32,
}
impl Future for Mock {
    type Output = ();

    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        if self.n == 0 {
            Poll::Ready(())
        } else {
            self.n -= 1;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
fn main() {
    block_on::block_on::block_on(Mock { n: 10 });
}
