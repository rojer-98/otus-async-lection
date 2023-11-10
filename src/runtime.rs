use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};

pub struct Task {
    pub inner: Pin<Box<dyn Future<Output = ()> + 'static>>,
}

pub struct Runtime {
    tasks: Vec<Option<Task>>,
}

impl Runtime {
    fn work_on(&mut self, fut: impl Future<Output = ()> + 'static) {
        let inner = Box::pin(fut);
        let task = Task { inner };

        self.tasks.push(Some(task));
    }

    fn process(&mut self) {
        loop {
            for maybe_ready in self.tasks.iter_mut() {
                if let Some(mut t) = maybe_ready.take() {
                    let waker = Waker::noop();
                    let mut cx = Context::from_waker(&waker);

                    if let Poll::Pending = t.inner.as_mut().poll(&mut cx) {
                        *maybe_ready = Some(t);
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
    let mut rt = Runtime { tasks: Vec::new() };

    rt.work_on(async {
        println!("I am async call");
    });
    rt.work_on(fut());

    rt.process();
}
