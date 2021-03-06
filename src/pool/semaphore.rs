use std::sync::Arc;
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};
use mco::std::queue::seg_queue::SegQueue;
use mco::std::sync::{Blocker, Semphore};
use crate::Error;
use crate::error::Result;

/// permit guard
pub struct PermitGuard<'a> {
    inner: &'a BoxSemaphore,
    blocker: Arc<mco::std::sync::Blocker>,
}

impl <'a>PermitGuard<'a>{
    pub fn release(self){
        self.inner.release();
    }
}

pub struct BoxSemaphore {
    /// permit total num
    total: i64,
    ///permit
    permit: AtomicI64,
    ///wait queue
    waiters: SegQueue<Arc<mco::std::sync::Blocker>>,
}

impl BoxSemaphore {
    pub fn new(size: usize) -> Self {
        Self {
            total: size as i64,
            permit: AtomicI64::new(size as i64),
            waiters: SegQueue::new(),
        }
    }

    pub fn permit(&self) -> i64 {
        self.permit.fetch_or(0, Ordering::Relaxed)
    }

    pub fn acquire(&self) -> PermitGuard {
        if self.permit() > 0 {
            self.permit.fetch_sub(1,Ordering::Relaxed);
            PermitGuard {
                inner: &self,
                blocker: Blocker::current(),
            }
        } else {
            let b = Blocker::current();
            self.waiters.push(b.clone());
            b.park(None);
            PermitGuard {
                inner: &self,
                blocker: b,
            }
        }
    }

    pub fn try_acquire(&self) -> Option<PermitGuard> {
        if self.permit() > 0 {
            Some(self.acquire())
        } else {
            None
        }
    }

    pub fn release(&self) {
        let per = self.permit();
        if per >= self.total {
            return;
        }
        if self.waiters.is_empty() {
            // If there are no waiters, just decrement and we're done
            self.permit.fetch_add(1,Ordering::Relaxed);
        } else {
            let w = self.waiters.pop();
            if let Some(w) = w {
                self.permit.fetch_add(1,Ordering::Relaxed);
                w.unpark();
            }
        }
    }

    pub fn release_left(&self, mut num: usize) -> usize {
        if self.permit() == self.total {
            return 0;
        }
        if num > self.total as usize {
            num = self.total as usize;
        }
        for _ in 0..num {
            self.release();
        }
        return num as usize;
    }
}


#[cfg(test)]
mod test {
    use std::sync::Arc;
    use std::time::Duration;
    use mco::coroutine::sleep;
    use mco::{chan, co};
    use crate::pool::semaphore::{BoxSemaphore};

    #[test]
    fn test_acq() {
        let b = Arc::new(BoxSemaphore::new(2));
        let b1 = b.clone();
        co!(move ||{
            b1.acquire();
            println!("{}",1);
            println!("num:{}",b1.permit());
        });
        sleep(Duration::from_secs(1));
        let b2 = b.clone();
        co!(move ||{
            b2.acquire();
            println!("{}",2);
            println!("num:{}",b2.permit());
        });
        sleep(Duration::from_secs(1));
        let b3 = b.clone();
        co!(move ||{
            println!("req b3");
            println!("num:{}",b3.permit());
            b3.acquire();
            println!("{}",3);
        });
        sleep(Duration::from_secs(1));
        let b4 = b.clone();
        co!(move ||{
            println!("release");
            b4.release();
            println!("num:{}",b4.permit());
        });
        sleep(Duration::from_secs(2));
    }


    #[test]
    fn test_acq_release_num() {
        let b = Arc::new(BoxSemaphore::new(3));
        println!("permit:{}", b.permit());
        b.acquire();
        b.acquire();
        b.acquire();

        println!("permit:{}", b.permit());

        let b1 = b.clone();
        co!(move ||{
            b1.acquire();
            println!("acq{}",4);
        });
        let b1 = b.clone();
        co!(move ||{
            b1.acquire();
            println!("acq{}",5);
        });
        let b1 = b.clone();
        co!(move ||{
            b1.acquire();
            println!("acq{}",6);
        });
        sleep(Duration::from_secs(1));
        println!("permit:{}", b.permit());
        b.release_left(2);
        println!("permit:{}", b.permit());
        sleep(Duration::from_secs(1));
    }

    #[test]
    fn test_acq_mult() {
        let total = 1000;
        let (s, r) = chan!();
        let b = Arc::new(BoxSemaphore::new(10));
        for idx in 0..total {
            let s1 = s.clone();
            let b1 = b.clone();
            let f1 = move || {
                let permit = b1.acquire();
                println!("acq{}", idx);
                s1.send(1);
                b1.release();
            };
            co!(f1);
        }
        let mut recvs = 0;
        for idx in 0..total {
            if let Ok(v) = r.recv() {
                recvs += 1;
            }
            if recvs == total {
                break;
            }
        }
    }
}