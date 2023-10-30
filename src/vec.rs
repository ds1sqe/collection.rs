#![allow(dead_code)]

use std::{
    alloc,
    alloc::Layout,
    isize,
    marker::PhantomData,
    mem::{self, ManuallyDrop},
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
};

struct RawVec<T> {
    ptr: NonNull<T>,
    cap: usize,
}
unsafe impl<T: Send> Send for RawVec<T> {}
unsafe impl<T: Sync> Sync for RawVec<T> {}

impl<T> RawVec<T> {
    fn new() -> Self {
        assert!(mem::size_of::<T>() != 0, "Cannot handle Zero sized types");
        RawVec {
            ptr: NonNull::dangling(),
            cap: 0,
        }
    }
    fn grow(&mut self) {
        let new_cap = if self.cap == 0 { 1 } else { 2 * self.cap };

        let new_layout = Layout::array::<T>(new_cap).unwrap();

        assert!(
            new_layout.size() <= isize::MAX as usize,
            "Too large to allocate"
        );

        let new_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
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
        if self.cap != 0 {
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}

pub struct Vec<T> {
    buf: RawVec<T>,
    len: usize,
}

impl<T> Vec<T> {
    fn ptr(&self) -> *mut T {
        self.buf.ptr.as_ptr()
    }

    fn cap(&self) -> usize {
        self.buf.cap
    }

    pub fn new() -> Self {
        Vec {
            buf: RawVec::new(),
            len: 0,
        }
    }

    fn grow(&mut self) {
        self.buf.grow();
    }

    pub fn push(&mut self, el: T) {
        if self.len == self.cap() {
            self.grow();
        }
        unsafe { ptr::write(self.ptr().add(self.len), el) }
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(ptr::read(self.ptr().add(self.len))) }
        }
    }

    pub fn insert(&mut self, index: usize, el: T) {
        assert!(
            index <= self.len,
            "Index out of bounds index:{index} > len:{}",
            self.len
        );

        if self.cap() == self.len {
            self.grow();
        }

        unsafe {
            // move elements after index to make room for el
            ptr::copy(
                self.ptr().add(index),
                self.ptr().add(index + 1),
                self.len - index,
            );
            ptr::write(self.ptr().add(index), el);
            self.len += 1;
        }
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(
            index <= self.len,
            "Index out of bounds index:{index} > len:{}",
            self.len
        );
        unsafe {
            self.len -= 1;
            let el = ptr::read(self.ptr().add(index));
            ptr::copy(
                self.ptr().add(index + 1),
                self.ptr().add(index),
                self.len - index,
            );
            el
        }
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop() {}
    }
}

impl<T> Deref for Vec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr(), self.len) }
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr(), self.len) }
    }
}

pub struct _IntoIter<T> {
    buf: RawVec<T>,
    start: *const T,
    end: *const T,
}

impl<T> Iterator for _IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let item = ptr::read(self.start);
                self.start = self.start.offset(1);
                Some(item)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.end as usize - self.start as usize) / mem::size_of::<T>();
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for _IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                self.end = self.end.offset(-1);
                Some(ptr::read(self.end))
            }
        }
    }
}

impl<T> Drop for _IntoIter<T> {
    fn drop(&mut self) {
        for _ in &mut *self {}
    }
}

impl<T> IntoIterator for Vec<T> {
    type Item = T;
    type IntoIter = _IntoIter<T>;

    fn into_iter(self) -> _IntoIter<T> {
        unsafe {
            let buf = ptr::read(&self.buf);
            let len = self.len;
            mem::forget(self);

            _IntoIter {
                start: buf.ptr.as_ptr(),
                end: if buf.cap == 0 {
                    buf.ptr.as_ptr()
                } else {
                    buf.ptr.as_ptr().add(len)
                },
                buf,
            }
        }
    }
}

struct _Drain<'a, T: 'a> {
    vec: PhantomData<&'a mut Vec<T>>,
    start: *const T,
    end: *const T,
}

impl<'a, T> Iterator for _Drain<'a, T> {}
