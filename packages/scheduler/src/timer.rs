use std::cmp::Ordering;
use std::future::Future;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::task::{Poll, Waker};
use std::time::{Duration, Instant};

use crate::scheduler::{TimerScheduler};

pub struct Timeout {
    at: Instant,
    is_completed: AtomicBool,
    waker: Option<Waker>,
}
impl Eq for Timeout {}

impl PartialEq for Timeout {
    fn eq(&self, other: &Self) -> bool {
        self.at.eq(&other.at)
    }
}
// runtime::Timeout(duration) { => internal::Timeout::new(timeout,scheduler)
// => scheduler.register(TimeoutCmd {timeout, callback });
// } =>   Timeout}.await
// => Future::poll(timeout,ctx)

impl Timeout {
    pub fn after(duration: Duration, scheduler: &TimerScheduler) -> Arc<Self> {
        let now = Instant::now();
        let it = Arc::new(Self {
            at: now + duration,
            is_completed: AtomicBool::new(false),
            waker: None,
        });
        let that = it.clone();
        let callback = Box::new(move || {
            that.is_completed.store(true, std::sync::atomic::Ordering::SeqCst);
            if let Some(waker) = &that.waker {
                waker.clone().wake();
            }
            println!("on callback!");
        });
        let that = it.clone();
        scheduler.register(that, callback);
        it
    }
    //pub fn at(instant:Instant) ->Self {
    //  Self {at:instant,is_completed: AtomicBool::new(instant < Instant::now())}
    //}
    pub fn remains(&self) -> Duration {
        let now = Instant::now();
        if now < self.at {
            self.at - now
        } else {
            Duration::ZERO
        }
    }
}
impl Ord for Timeout {
    fn cmp(&self, other: &Self) -> Ordering {
        other.at.cmp(&self.at)
    }
}
impl PartialOrd for Timeout {
    fn partial_cmp(&self, other: &Timeout) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Future for Timeout {
    type Output = ();

    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        match self.is_completed.load(std::sync::atomic::Ordering::SeqCst) {
            true => Poll::Ready(()),
            false => {
                self.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}
