use std::sync::atomic::{fence, AtomicBool, Ordering};

pub struct Mutex {
    flag: AtomicBool,
}

impl Mutex {
    pub fn new() -> Self {
        Self {
            flag: AtomicBool::new(false),
        }
    }

    pub fn lock(&self) {
        // Wait until the flag to be false
        while self
            .flag
            .compare_exchange_weak(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {}

        // synchronizes-with release-store in fn `unlock`
        fence(Ordering::Acquire)
    }

    pub fn unlock(&self) {
        self.flag.store(false, Ordering::Release);
    }
}
