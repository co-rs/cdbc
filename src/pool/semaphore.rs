use may::sync::Semphore;
use crate::Error;
use crate::error::Result;

pub struct Semaphore {
    inner: may::sync::Semphore,
}

impl Semaphore {
    pub fn new(capacity:usize) -> Semaphore {
        Self{
            inner:Semphore::new(capacity)
        }
    }

    pub fn try_acquire(&self) -> Option<usize> {
        if self.inner.get_value()<=0{
            return None;
        }else{
            self.inner.post();
        }
        return Some(1);
    }
    pub fn acquire(&self) -> usize {
        self.inner.post();
        return 1;
    }
    pub fn acquire_num(&self,num:usize) -> usize {
        for _ in 0..num{
            self.inner.post();
        }
        return num;
    }
    pub fn release(&self) -> usize{
        self.inner.wait();
        return 1;
    }

    pub fn try_release(&self) -> usize {
        let success=self.inner.try_wait();
        if success == false{
            return 0;
        }
        return 1;
    }

    pub fn release_left(&self, num: usize) -> usize {
        let left = self.inner.get_value();
        if left >= num {
            for _ in 0..num {
                self.inner.wait();
            }
            return num;
        }else{
            for _ in 0..left {
                self.inner.wait();
            }
            return left;
        }
    }
}