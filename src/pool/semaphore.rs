use std::sync::Arc;
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};
use may::sync::{Blocker, Semphore};
use crate::Error;
use crate::error::Result;

pub struct BoxSemaphore {
    inner: may::sync::Semphore,
}

impl BoxSemaphore {
    pub fn new(capacity: usize) -> BoxSemaphore {
        let s = Self {
            inner: Semphore::new(capacity)
        };
        for _ in 0..capacity {
            s.inner.post();
        }
        s
    }

    pub fn try_acquire(&self) -> Option<usize> {
        if self.inner.try_wait() {
            return Some(1);
        }
        return None;
    }
    pub fn acquire(&self) -> usize {
        self.inner.wait();
        return 1;
    }
    pub fn acquire_num(&self, num: usize) -> usize {
        for _ in 0..num {
            self.inner.post();
        }
        self.inner.wait();
        return num;
    }
    pub fn release(&self) -> usize {
        self.inner.post();
        return 1;
    }

    pub fn try_release(&self) -> usize {
        self.inner.post();
        return 1;
    }

    pub fn release_left(&self, num: usize) -> usize {
        let left = self.inner.get_value();
        if left >= num {
            for _ in 0..num {
                self.inner.post();
            }
            return num;
        } else {
            for _ in 0..left {
                self.inner.post();
            }
            return left;
        }
    }
}


pub struct BoxSemaphore2 {
    size: i64,
    cur: AtomicI64,
    waiters: crossbeam_queue::SegQueue<Arc<may::sync::Blocker>>,
}


impl BoxSemaphore2 {
    pub fn new(size: usize) -> Self {
        Self {
            size: size as i64,
            cur: AtomicI64::new(0),
            waiters: crossbeam_queue::SegQueue::new(),
        }
    }

    pub fn cur(&self) -> i64 {
        self.cur.fetch_or(0, Ordering::Relaxed)
    }

    pub fn acq(&self) -> Option<Arc<Blocker>> {
        if self.cur() < self.size {
            self.cur.fetch_add(1, Ordering::Relaxed);
            None
        } else {
            let b = Blocker::current();
            b.park(None);
            self.waiters.push(b.clone());
            Some(b)
        }
    }

    pub fn release(&self) {
        if self.cur() < 1 {
            return;
        }
        if self.waiters.is_empty(){
            // If there are no waiters, just decrement and we're done
            self.cur.fetch_sub(1,Ordering::Relaxed);
        }

        let w=self.waiters.pop();
        if let Some(w) = w{
            w.unpark();
        }
    }
}


#[cfg(test)]
mod test {
    use std::sync::Arc;
    use std::thread::sleep;
    use std::time::Duration;
    use may::go;
    use crate::pool::semaphore::{BoxSemaphore, BoxSemaphore2};

    #[test]
    fn test_acq() {
        let b = Arc::new(BoxSemaphore2::new(2));
        let b1 = b.clone();
        go!(move ||{
            b1.acq();
            println!("{}",1);
        });
        sleep(Duration::from_secs(1));
        let b2 = b.clone();
        go!(move ||{
            b2.acq();
            println!("{}",2);
        });
        sleep(Duration::from_secs(1));
        let b3 = b.clone();
        go!(move ||{
            b3.release();
            b3.acq();
            println!("{}",3);
        });

        sleep(Duration::from_secs(2));
    }
}