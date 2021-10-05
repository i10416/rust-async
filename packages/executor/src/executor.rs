use std::{sync::Arc, thread};

use crossbeam_channel;
use crossbeam_channel::Sender;
use num_cpus;
use once_cell::sync::Lazy;
struct RunnableTask {
}

impl RunnableTask {
    fn run(self:Arc<RunnableTask>) {
    }
}

static QUEUE: Lazy<Sender<Arc<RunnableTask>>> = Lazy::new(|| {
    let (tx, rx) = crossbeam_channel::unbounded::<Arc<RunnableTask>>();
    (0..num_cpus::get().max(1)).for_each(|_| {
        let rx = rx.clone();
        thread::spawn(move || rx.iter().for_each(|task| task.run()));
    });
    tx
});
