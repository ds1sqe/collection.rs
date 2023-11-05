#![allow(dead_code)]

use std::{alloc, alloc::Layout, mem, ptr::NonNull};
#[derive(Debug, Clone)]
pub struct RawVec<T> {
    pub ptr: NonNull<T>,
    pub cap: usize,
}
unsafe impl<T: Send> Send for RawVec<T> {}
unsafe impl<T: Sync> Sync for RawVec<T> {}

impl<T> RawVec<T> {
    pub fn new() -> Self {
        let cap = if mem::size_of::<T>() == 0 {
            // if T is zero sized type
            usize::MAX
        } else {
            // else
            0
        };

        RawVec {
            ptr: NonNull::dangling(),
            cap,
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        let mut real_cap = 1;
        while real_cap < cap {
            real_cap *= 2;
        }

        let layout = Layout::array::<T>(real_cap).unwrap();
        assert!(
            layout.size() <= isize::MAX as usize,
            "Too large to allocate"
        );

        let new_ptr = unsafe { alloc::alloc(layout) };

        let ptr = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => alloc::handle_alloc_error(layout),
        };

        RawVec { ptr, cap: real_cap }
    }

    pub fn grow(&mut self) {
        // since when size of T is 0, capacity was setted to usize::MAX
        assert!(mem::size_of::<T>() != 0, "capacity overflow");

        let (new_cap, new_layout) = if self.cap == 0 {
            (1, Layout::array::<T>(1).unwrap())
        } else {
            let new_cap = 2 * self.cap;
            let new_layout = Layout::array::<T>(new_cap).unwrap();
            (new_cap, new_layout)
        };

        assert!(
            new_layout.size() <= isize::MAX as usize,
            "Too large to allocate"
        );

        let new_ptr = if self.cap == 0 {
            // allocate layout with global allocator
            unsafe { alloc::alloc(new_layout) }
        } else {
            // reallocate layout with global allocator
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe { alloc::realloc(old_ptr, old_layout, new_layout.size()) }
        };

        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout),
        };
        self.cap = new_cap
    }
}
impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        let is_zst = mem::size_of::<T>() == 0;
        if self.cap != 0 && !is_zst {
            unsafe {
                alloc::dealloc(
                    self.ptr.as_ptr() as *mut u8,
                    Layout::array::<T>(self.cap).unwrap(),
                );
            }
        }
    }
}
