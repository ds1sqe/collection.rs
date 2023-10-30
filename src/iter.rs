use std::{mem, ptr};

pub struct RawIter<T> {
    start: *const T,
    end: *const T,
}

impl<T> RawIter<T> {
    pub unsafe fn new(slice: &[T]) -> Self {
        RawIter {
            start: slice.as_ptr(),
            end: if slice.len() == 0 {
                slice.as_ptr()
            } else {
                slice.as_ptr().add(slice.len())
            },
        }
    }

    pub fn next(&mut self) -> Option<T> {
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

    pub fn next_back(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                self.end = self.end.offset(-1);
                Some(ptr::read(self.end))
            }
        }
    }

    pub fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.end as usize - self.start as usize) / mem::size_of::<T>();
        (len, Some(len))
    }
}
