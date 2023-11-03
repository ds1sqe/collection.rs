use std::{
    mem,
    ptr::{self, NonNull},
};

pub struct RawIter<T> {
    start: *const T,
    end: *const T,
}

impl<T> RawIter<T> {
    pub unsafe fn new(slice: &[T]) -> Self {
        RawIter {
            start: slice.as_ptr(),
            end: if mem::size_of::<T>() == 0 {
                // if T is zst, cast pointer to usize, increment, and then
                // cast it back
                ((slice.as_ptr() as usize) + slice.len()) as *const _
            } else if slice.len() == 0 {
                slice.as_ptr()
            } else {
                slice.as_ptr().add(slice.len())
            },
        }
    }
}
impl<T> Iterator for RawIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                if mem::size_of::<T>() == 0 {
                    self.start = (self.start as usize + 1) as *const _;
                    Some(ptr::read(NonNull::<T>::dangling().as_ptr()))
                } else {
                    let old = self.start;
                    self.start = self.start.offset(1);
                    Some(ptr::read(old))
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = mem::size_of::<T>();
        let len = (self.end as usize - self.start as usize) / if size == 0 { 1 } else { size };
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for RawIter<T> {
    fn next_back(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                if mem::size_of::<T>() == 0 {
                    self.end = (self.end as usize - 1) as *const _;
                    Some(ptr::read(NonNull::<T>::dangling().as_ptr()))
                } else {
                    self.end = self.end.offset(-1);
                    Some(ptr::read(self.end))
                }
            }
        }
    }
}
