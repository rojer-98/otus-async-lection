use std::{
    future::Future,
    mem::ManuallyDrop,
    pin::Pin,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use crossbeam_channel::{unbounded, Receiver, Sender};

impl<T: Wakeable> WakeableTraitObject for T {}

fn w_cast<T: Wakeable>(w: *const ()) -> Arc<T> {
    let w_c = w.cast::<T>();
    let new_arc = unsafe { Arc::from_raw(w_c) };

    new_arc
}

trait WakeableTraitObject: Wakeable + Sized {
    fn into_raw_waker(self: Arc<Self>) -> RawWaker {
        let r_w = Arc::into_raw(self).cast::<()>();
        let r_w_vt = &Self::WAKER_VTABLE;

        RawWaker::new(r_w, r_w_vt)
    }

    const WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
        {
            unsafe fn clone<T: Wakeable>(w: *const ()) -> RawWaker {
                let arc = w_cast::<T>(w);
                let arc_without_drop = ManuallyDrop::new(arc);

                Arc::clone(&arc_without_drop).into_raw_waker()
            }

            clone::<Self>
        },
        {
            unsafe fn wake<T: Wakeable>(w: *const ()) {
                let arc = w_cast::<T>(w);

                Wakeable::wake(arc);
            }

            wake::<Self>
        },
        {
            unsafe fn wake_by_ref<T: Wakeable>(w: *const ()) {
                let arc = w_cast::<T>(w);
                let arc_without_drop = ManuallyDrop::new(arc);

                Wakeable::wake_by_ref(&arc_without_drop);
            }

            wake_by_ref::<Self>
        },
        {
            unsafe fn drop<T: Wakeable>(w: *const ()) {
                use std::mem::drop as mem_drop;
                let arc = w_cast::<T>(w);

                mem_drop(arc);
            }

            drop::<Self>
        },
    );
}

pub trait Wakeable: Sized {
    fn wake(self: Arc<Self>) {
        Self::wake_by_ref(&self);
    }

    fn wake_by_ref(self: &'_ Arc<Self>);

    fn into_waker(self: &'_ Arc<Self>) -> Waker {
        unsafe { Waker::from_raw(Self::into_raw_waker(Arc::clone(self))) }
    }
}

pub struct Task {
    pub inner: Mutex<Pin<Box<dyn Future<Output = ()> + 'static>>>,
    tx: Sender<Arc<Task>>,
}

impl Wakeable for Task {
    fn wake_by_ref(self: &'_ Arc<Self>) {
        if let Err(e) = self.tx.send(Arc::clone(self)) {
            println!("Cannot send task: Error: {e}");
        }
    }
}

pub struct Runtime {
    tx: Sender<Arc<Task>>,
    rx: Receiver<Arc<Task>>,
}

impl Default for Runtime {
    fn default() -> Self {
        let (tx, rx) = unbounded();

        Self { rx, tx }
    }
}

impl Runtime {
    fn work_on(&mut self, fut: impl Future<Output = ()> + 'static) {
        let task = Arc::new(Task {
            inner: Mutex::new(Box::pin(fut)),
            tx: self.tx.clone(),
        });

        if let Err(e) = self.tx.send(task) {
            println!("Cannot send task. Error: {e}");
        }
    }

    fn process(&mut self) {
        loop {
            while let Ok(task) = self.rx.recv() {
                let waker = Wakeable::into_waker(&task);
                let mut cx = Context::from_waker(&waker);
                let mut t = task.inner.lock();

                if let Ok(ref mut i_t) = t {
                    if let Poll::Pending = i_t.as_mut().poll(&mut cx) {
                        drop(t)
                    }
                }
            }
        }
    }
}

struct GiveNumberFuture {
    give_after_tries: u32,
    current_tries: u32,
}

impl Future for GiveNumberFuture {
    type Output = u32;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        println!("polled {} times", this.current_tries + 1);

        if this.give_after_tries > this.current_tries + 1 {
            this.current_tries += 1;
            cx.waker().wake_by_ref();

            Poll::Pending
        } else {
            Poll::Ready(20)
        }
    }
}

async fn fut() {
    let fut = GiveNumberFuture {
        give_after_tries: 10,
        current_tries: 0,
    };

    let number = fut.await;

    println!("waited for {number}");
}

pub fn runtime_main() {
    let mut rt = Runtime::default();
    let counter = AtomicU64::new(0);

    rt.work_on(fut());
    rt.work_on(async move {
        for _ in 0.. {
            counter.fetch_add(1, Ordering::Relaxed);

            async {
                let c = counter.load(Ordering::Relaxed);

                println!("Counter is {c}");
            }
            .await
        }
    });

    rt.work_on(async {
        println!("I am async call");
    });
    rt.work_on(fut());

    rt.process();
}
