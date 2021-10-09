use crate::timer::Timeout;
use std::{
    borrow::BorrowMut,
    cell::RefCell,
    collections::BinaryHeap,
    sync::{Arc, Mutex},
    thread::JoinHandle,
};
pub struct TimerReactor {
    tree: RefCell<BinaryHeap<TimeoutCmd>>,
    handle: Option<JoinHandle<()>>,
}

pub struct TimeoutCmd {
    timeout: Arc<Timeout>,
    callback: Box<dyn Fn() + Send>,
}
impl Eq for TimeoutCmd {}
impl PartialEq for TimeoutCmd {
    fn eq(&self, other: &Self) -> bool {
        self.timeout.eq(&other.timeout)
    }
}
impl TimeoutCmd {
    pub fn new(timeout: Arc<Timeout>, callback: Box<dyn Fn() + Send>) -> Self {
        Self {
            timeout: timeout,
            callback: callback,
        }
    }
}
impl PartialOrd for TimeoutCmd {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.timeout.partial_cmp(&other.timeout)
    }
}
impl Ord for TimeoutCmd {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.timeout.cmp(&other.timeout)
    }
}

// struct Runtime has a timer scheduler, impl Timeout(timeout) func.
// expected implementation: runtime::Timeout(timeout) => internal::Timeout::new { timeout,&scheduler }
impl TimerReactor {
    pub fn new() -> Arc<Mutex<Self>> {
        let scheduler = Arc::new(Mutex::new(Self {
            tree: RefCell::new(BinaryHeap::new()),
            handle: None,
        }));
        let res = scheduler.clone();
        let th = std::thread::spawn(move || loop {
            if let Some(TimeoutCmd { timeout, callback }) = scheduler.lock().map(|it| it.tree.take().pop()).unwrap() {
                std::thread::park_timeout(timeout.remains());
                callback();
            }
        });
        res.lock().map(|mut it| it.handle = Some(th)).unwrap();
        res
    }
    pub fn register(&self, timeout: Arc<Timeout>, callback: Box<dyn Fn() + Send + 'static>) {
        if timeout.remains().is_zero() {
            let cmd = TimeoutCmd {
                timeout: timeout,
                callback: callback,
            };
            // todo: handle runtime contention. it would be better to relpace logic to use channel.
            self.tree.borrow_mut().push(cmd);
        }
    }
}

impl Drop for TimerReactor {
    fn drop(&mut self) {
        self.handle.take().map(|it| it.join().unwrap()).unwrap();
    }
}

#[cfg(test)]
mod scheduler_test {
    use std::time::Duration;

    use crate::timer::Timeout;

    use super::TimerReactor;

    #[test]
    fn test() {
        let scheduler = TimerReactor::new();
        //let it = scheduler.clone();
        /*std::thread::spawn(move|| {
          Timeout::after(Duration::from_secs(1), &scheduler);
        });
        std::thread::spawn( move|| {
          Timeout::after(Duration::from_secs(2), &it);
        }); */
    }
}
