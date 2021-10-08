use crate::timer::Timeout;
use std::{
    collections::BinaryHeap,
    sync::{Arc, Mutex},
    thread::JoinHandle,
};
pub struct TimerScheduler {
    tree: Mutex<BinaryHeap<TimeoutCmd>>,
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
  pub fn new(timeout:Arc<Timeout>,callback:Box<dyn Fn() + Send>) ->Self{
    Self {
      timeout:timeout,
      callback:callback
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
impl TimerScheduler {
    pub fn new() -> (JoinHandle<()>, Arc<Self>) {
        let scheduler = Arc::new(Self {
            tree: Mutex::new(BinaryHeap::new()),
        });
        let res = scheduler.clone();
        let th = std::thread::spawn(move || loop {
            if let Some(TimeoutCmd { timeout, callback }) = scheduler.tree.lock().unwrap().pop() {
                std::thread::park_timeout(timeout.remains());
                callback();
            }
        });
        (th, res)
    }
    pub fn register(&self, timeout:Arc<Timeout>,callback:Box<dyn Fn()+Send+'static>) {
        if timeout.remains().is_zero(){
        let cmd = TimeoutCmd {timeout:timeout,callback:callback};
        self.tree.lock().unwrap().push(cmd);
        }
    }
}


#[cfg(test)]
mod scheduler_test {
    use std::time::Duration;

    use crate::timer::Timeout;

    use super::{TimerScheduler};

    #[test]
    fn test() {
        let (handle, scheduler) = TimerScheduler::new();
        let it = scheduler.clone();
        std::thread::spawn(move|| {
          Timeout::after(Duration::from_secs(1), &scheduler);
        });
        std::thread::spawn( move|| {
          Timeout::after(Duration::from_secs(2), &it);
        });
        handle.join().unwrap();
    }
}
