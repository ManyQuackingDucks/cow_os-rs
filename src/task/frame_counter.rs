extern crate conquer_once;
use crate::println;
use crate::FRAMECOUNTER;
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::stream::Stream;
use futures_util::stream::StreamExt;
use futures_util::task::AtomicWaker;

static FRAMEQUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

pub(crate) fn frame_count() {
    if let Ok(queue) = FRAMEQUEUE.try_get() {
        let mut a = FRAMECOUNTER.lock();
        if queue.push(*a).is_err() {
            println!("WARNING: frame queue full");
        } else {
            WAKER.wake();
        }
        *a = 0;
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

pub struct FrameCountStream {
    _private: (),
}

impl FrameCountStream {
    pub fn new() -> Self {
        FRAMEQUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("FrameCountStream::new should only be called once");
        FrameCountStream { _private: () }
    }
}

impl Default for FrameCountStream {
    fn default() -> Self {
        Self::new()
    }
}

impl Stream for FrameCountStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = FRAMEQUEUE.try_get().expect("Frame queue not initialized");

        // fast path
        if let Some(frame) = queue.pop() {
            return Poll::Ready(Some(frame));
        }

        WAKER.register(cx.waker());
        match queue.pop() {
            Some(frame) => {
                WAKER.take();
                Poll::Ready(Some(frame))
            }
            None => Poll::Pending,
        }
    }
}

pub async fn print_frames() {
    let mut framecounts = FrameCountStream::new();
    while let Some(framecount) = framecounts.next().await {
        println!("Frame count: {}", framecount);
    }
}
