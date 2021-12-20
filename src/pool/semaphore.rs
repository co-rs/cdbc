use std::sync::Arc;
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};
use may::sync::{Blocker, Semphore};
use crate::Error;
use crate::error::Result;

pub struct BoxSemaphore {
    /// permit total num
    total: i64,
    ///permit
    permit: AtomicI64,
    ///wait queue
    waiters: crossbeam_queue::SegQueue<Arc<may::sync::Blocker>>,
}

impl BoxSemaphore {
    pub fn new(size: usize) -> Self {
        Self {
            total: size as i64,
            permit: AtomicI64::new(0 as i64),
            waiters: crossbeam_queue::SegQueue::new(),
        }
    }

    pub fn permit(&self) -> i64 {
        self.permit.fetch_or(0, Ordering::Relaxed)
    }

    pub fn acquire(&self) -> Arc<Blocker> {
        if self.permit() < self.total {
            self.permit.fetch_add(1, Ordering::Relaxed);
            Blocker::current()
        } else {
            let b = Blocker::current();
            self.waiters.push(b.clone());
            b.park(None);
            b
        }
    }

    pub fn try_acquire(&self) -> Option<Arc<Blocker>> {
        if self.permit() < self.total {
            Some(self.acquire())
        } else {
            None
        }
    }

    pub fn release(&self) {
        if self.permit() == 0 {
            return;
        }
        if self.waiters.is_empty() {
            // If there are no waiters, just decrement and we're done
            self.permit.fetch_sub(1, Ordering::Relaxed);
        } else {
            let w = self.waiters.pop();
            if let Some(w) = w {
                self.permit.fetch_sub(1, Ordering::Relaxed);
                w.unpark();
            }
        }
    }

    pub fn release_left(&self, mut num: usize) -> usize {
        if self.permit() == 0 {
            return 0;
        }
        if num > self.permit() as usize {
            num = self.permit() as usize;
        }
        if self.waiters.is_empty() {
            self.permit.fetch_sub(num as i64, Ordering::Relaxed);
            return num;
        } else {
            for _ in 0..num {
                self.release();
            }
            return num as usize;
        }
    }
}


#[cfg(test)]
mod test {
    use std::sync::Arc;
    use std::time::Duration;
    use may::coroutine::sleep;
    use may::go;
    use crate::pool::semaphore::{BoxSemaphore};

    #[test]
    fn test_acq() {
        let b = Arc::new(BoxSemaphore::new(2));
        let b1 = b.clone();
        go!(move ||{
            b1.acquire();
            println!("{}",1);
            println!("num:{}",b1.permit());
        });
        sleep(Duration::from_secs(1));
        let b2 = b.clone();
        go!(move ||{
            b2.acquire();
            println!("{}",2);
            println!("num:{}",b2.permit());
        });
        sleep(Duration::from_secs(1));
        let b3 = b.clone();
        go!(move ||{
            println!("req b3");
            println!("num:{}",b3.permit());
            b3.acquire();
            println!("{}",3);
        });
        sleep(Duration::from_secs(1));
        let b4 = b.clone();
        go!(move ||{
            println!("release");
            b4.release();
            println!("num:{}",b4.permit());
        });
        sleep(Duration::from_secs(2));
    }


    #[test]
    fn test_acq_release_num() {
        let b = Arc::new(BoxSemaphore::new(3));
        let b1 = b.clone();
        go!(move ||{
            b1.acquire();
            println!("acq{}",1);
        });
        let b1 = b.clone();
        go!(move ||{
            b1.acquire();
            println!("acq{}",2);
        });
        let b1 = b.clone();
        go!(move ||{
            b1.acquire();
            println!("acq{}",3);
        });

        sleep(Duration::from_secs(1));
        println!("num:{}",b.permit());

        let b1 = b.clone();
        go!(move ||{
            b1.acquire();
            println!("acq{}",4);
        });
        let b1 = b.clone();
        go!(move ||{
            b1.acquire();
            println!("acq{}",5);
        });
        let b1 = b.clone();
        go!(move ||{
            b1.acquire();
            println!("acq{}",6);
        });
        sleep(Duration::from_secs(1));
        println!("num:{}",b.permit());
        b.release_left(2);
        sleep(Duration::from_secs(1));
    }
}