use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};
use core::cell::UnsafeCell;


// FIXME: The Rustonomicon has a passage about poisoning data during a panic
// unwind...

pub struct Mutex<T> {
    lock: AtomicBool,
    item: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    pub const fn new(item: T) -> Self {
        Self { lock: AtomicBool::new(false), item: UnsafeCell::new(item) }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        while !self.lock.compare_and_swap(false, true, Ordering::SeqCst) {
            // Need to find a riscv equivalent...
            //unsafe { asm!("pause" :::: "volatile"); }
        }
        MutexGuard { lock: &self.lock, item: unsafe { &mut *self.item.get() } }
    }
    pub unsafe fn bust_lock(&mut self) {
        self.unlock();
    }
    fn unlock(&mut self) {
        self.lock.store(false, Ordering::SeqCst);
    }
}

pub struct MutexGuard<'a, T> {
    lock: &'a AtomicBool,
    item: &'a mut T,
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::SeqCst);
    }
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.item
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.item
    }
}

unsafe impl<T> Sync for Mutex<T> {}