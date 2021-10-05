use async_task::{self};
use crossbeam_channel;
use crossbeam_channel::Sender;
use num_cpus;
use once_cell::sync::Lazy;
use std::future::Future;
use std::panic::catch_unwind;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::task::{Context, Poll};
use std::{sync::Arc, thread};
use futures::channel::oneshot;
struct RunnableTask<T> {
    state: AtomicUsize,
    // 或るスレッドで生成したfuture を executor に共有するために Mutex<Pin<Box>>を使う.
    future: Mutex<Pin<Box<dyn Future<Output = T> + Send>>>,
}
/**
 * expected something like this
 * fn main() { futures::executor::block_on(async { let handle = spawn(async { 1 + 2 }); assert_eq!(handle.await, 3); }); }
 */
type JoinHandle<R> = Pin<Box<dyn Future<Output = R> + Send>>;


const WOKEN: usize = 0b01;
const RUNNING: usize = 0b10;
// spawn: future -> FiberHandle
fn spawn<F: Future<Output=R>+Send+'static,R:Send + 'static> (future:F)->JoinHandle<R> {
  let (tx,rx) = oneshot::channel::<R>();
  // create task
  // このタスクはwaker callback: =>Unit を 呼べばいいので Output = () でいい.
  
  // Future は cats-effect の IO のように インスタンスの生成と実行が分離されているので合成できる.
  let future = async {
    let f = future.await;
    let _ = tx.send(f);
  };
  let task = Arc::new(RunnableTask::<()>{future:Mutex::new(Box::pin(future)),state:AtomicUsize::new(0)});
  // send task to executor
  QUEUE.send(task).unwrap();
  Box::pin(async { rx.await.unwrap() })
}

impl RunnableTask<()> {
    fn run(self: Arc<RunnableTask<()>>) {
        let task = self.clone();
        // taskの実行 = future の poll
        let waker = async_task::waker_fn(move || {
            if task.state.fetch_or(WOKEN, Ordering::SeqCst) == 0 {
                QUEUE.send(task.clone()).unwrap();
            }
        });
        let cx = &mut Context::from_waker(&waker);
        self.state.store(RUNNING, Ordering::SeqCst);
        let poll = self.future.try_lock().unwrap().as_mut().poll(cx);
        if poll.is_pending() {
            if self.state.fetch_and(!RUNNING, Ordering::SeqCst) == WOKEN | RUNNING {
                QUEUE.send(self.clone()).unwrap()
            }
        }
    }
}

static QUEUE: Lazy<Sender<Arc<RunnableTask<()>>>> = Lazy::new(|| {
    let (tx, rx) = crossbeam_channel::unbounded::<Arc<RunnableTask<()>>>();
    (0..num_cpus::get().max(1)).for_each(|_| {
        let rx = rx.clone();
        thread::spawn(move || rx.iter().for_each(|task| {
          let _  = catch_unwind(|| task.run());
        }));
    });
    tx
});
