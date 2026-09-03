use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex};

pub struct Ring {
    inner: Mutex<VecDeque<f32>>,
    cond: Condvar,
    capacity: usize,
    done: AtomicBool,
}

impl Ring {
    pub fn new(capacity: usize) -> Ring {
        Ring {
            inner: Mutex::new(VecDeque::with_capacity(capacity)),
            cond: Condvar::new(),
            capacity,
            done: AtomicBool::new(false),
        }
    }

    pub fn space(&self) -> usize {
        self.capacity - self.inner.lock().unwrap().len()
    }

    pub fn mark_done(&self) {
        self.done.store(true, Ordering::Release);
        self.cond.notify_all();
    }

    pub fn push(&self, data: &[f32]) -> usize {
        let mut queue = self.inner.lock().unwrap();
        let room = self.capacity - queue.len();
        let count = room.min(data.len());
        for &sample in data.iter().take(count) {
            queue.push_back(sample);
        }
        self.cond.notify_one();
        count
    }

    pub fn wait_push(&self, data: &[f32]) -> usize {
        let mut queue = self.inner.lock().unwrap();
        while queue.len() >= self.capacity {
            queue = self.cond.wait(queue).unwrap();
        }
        drop(queue);
        self.push(data)
    }

    pub fn pop(&self, out: &mut [f32]) -> usize {
        let mut queue = self.inner.lock().unwrap();
        let count = queue.len().min(out.len());
        for slot in out.iter_mut().take(count) {
            *slot = queue.pop_front().unwrap();
        }
        count
    }
}
