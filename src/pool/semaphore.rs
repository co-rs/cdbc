use std::sync::Arc;
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};
use may::sync::{Blocker, Semphore};
use crate::Error;
use crate::error::Result;

pub struct BoxSemaphore {
    limit: i64,
    cur: AtomicI64,
    waiters: crossbeam_queue::SegQueue<Arc<may::sync::Blocker>>,
}

impl BoxSemaphore {
    pub fn new(size: usize) -> Self {
        Self {
            limit: size as i64,
            cur: AtomicI64::new(0 as i64),
            waiters: crossbeam_queue::SegQueue::new(),
        }
    }

    pub fn cur(&self) -> i64 {
        self.cur.fetch_or(0, Ordering::Relaxed)
    }

    pub fn acquire(&self) -> Arc<Blocker> {
        if self.cur() < self.limit {
            self.cur.fetch_add(1, Ordering::Relaxed);
            Blocker::current()
        } else {
            let b = Blocker::current();
            b.park(None);
            self.waiters.push(b.clone());
            b
        }
    }

    pub fn try_acquire(&self) -> Arc<Blocker> {
        if self.cur() < self.limit {
            self.cur.fetch_add(1, Ordering::Relaxed);
            Blocker::current()
        } else {
            let b = Blocker::current();
            self.waiters.push(b.clone());
            b
        }
    }

    pub fn release(&self) {
        if self.cur() == 0 {
            return;
        }
        if self.waiters.is_empty() {
            // If there are no waiters, just decrement and we're done
            self.cur.fetch_sub(1, Ordering::Relaxed);
        } else {
            let w = self.waiters.pop();
            if let Some(w) = w {
                w.unpark();
                self.cur.fetch_sub(1, Ordering::Relaxed);
            }
        }
    }

    pub fn release_left(&self, mut num: usize) -> usize {
        if self.cur() == 0 {
            return 0;
        }
        if num > self.cur() as usize {
            num = self.cur() as usize;
        }
        if self.waiters.is_empty() {
            self.cur.fetch_sub(num as i64, Ordering::Relaxed);
            return num;
        } else {
            let mut releases = 0;
            for _ in 0..num {
                let w = self.waiters.pop();
                if let Some(w) = w {
                    w.unpark();
                    releases += 1;
                }
            }
            releases = self.cur.fetch_sub(releases, Ordering::Relaxed);
            return releases as usize;
        }
    }
}


#[cfg(test)]
mod test {
    use std::sync::Arc;
    use std::thread::sleep;
    use std::time::Duration;
    use may::go;
    use crate::pool::semaphore::{BoxSemaphore};

    #[test]
    fn test_acq() {
        let b = Arc::new(BoxSemaphore::new(2));
        let b1 = b.clone();
        go!(move ||{
            b1.acquire();
            println!("{}",1);
        });
        sleep(Duration::from_secs(1));
        let b2 = b.clone();
        go!(move ||{
            b2.acquire();
            println!("{}",2);
        });
        sleep(Duration::from_secs(1));
        let b3 = b.clone();
        go!(move ||{
            b3.release();
            b3.acquire();
            println!("{}",3);
        });

        sleep(Duration::from_secs(2));
    }
}