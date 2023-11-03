#![allow(unused)]

use core::fmt;
use std::{
    collections::VecDeque,
    mem,
    ops::{Range, RangeBounds},
    ptr, slice,
};

use super::{raw::raw_vec::RawVec, vec::Vec};

struct Deque<T> {
    head: usize,
    len: usize,
    buf: RawVec<T>,
}

/// return index for logical index
fn wrap_idx(logical_idx: usize, cap: usize) -> usize {
    assert!((logical_idx == 0 && cap == 0) || logical_idx < cap || (logical_idx - cap) < cap);
    if logical_idx >= cap {
        logical_idx - cap
    } else {
        logical_idx
    }
}

impl<T> Deque<T> {
    pub fn new() -> Self {
        Self {
            head: 0,
            len: 0,
            buf: RawVec::new(),
        }
    }

    fn cap(&self) -> usize {
        self.buf.cap
    }

    fn grow(&mut self) {
        let old_cap = self.cap();
        self.buf.grow();
        unsafe {
            self.handle_grow(old_cap);
        }
    }

    fn handle_grow(&mut self, old_cap: usize) {
        // Move the shortest contiguous section of the ring buffer
        //
        // H := head
        // L := last element (`self.to_physical_idx(self.len - 1)`)
        //
        //    H           L
        //   [o o o o o o o . ]
        //    H           L
        // A [o o o o o o o . . . . . . . . . ]
        //
        //        L H
        //   [o o o o o o o o ]
        //          H           L
        // B [. . . o o o o o o o . . . . . . ]
        //
        //              L H
        //   [o o o o o o o o ]
        //            L                   H
        // C [o o o o o . . . . . . . . . o o ]

        if self.head <= old_cap - self.len {
            // Case A ) No op
        } else {
            let new_cap = self.cap();
            let head_len = old_cap - self.head;
            let tail_len = self.len - head_len;
            if head_len > tail_len && new_cap - old_cap >= tail_len {
                // Case B
                unsafe {
                    self.copy_nooverlap(0, old_cap, tail_len);
                }
            } else {
                // Case C

                let new_head = new_cap - head_len;

                // can't use copy_nonoverlapping here, because if e.g. head_len = 2
                // and new_capacity = old_capacity + 1, then the heads overlap.
                unsafe {
                    self.copy(self.head, new_head, head_len);
                }
                self.head = new_head;
            }
        }
    }

    fn is_full(&self) -> bool {
        self.len == self.cap()
    }

    fn is_empth(&self) -> bool {
        self.len == 0
    }

    fn wrap_sub(&self, idx: usize, sub: usize) -> usize {
        wrap_idx(idx.wrapping_sub(sub).wrapping_add(self.cap()), self.cap())
    }

    fn wrap_add(&self, idx: usize, add: usize) -> usize {
        wrap_idx(idx.wrapping_add(add), self.cap())
    }

    fn to_physical_idx(&self, idx: usize) -> usize {
        self.wrap_add(self.head, idx)
    }

    fn ptr(&self) -> *mut T {
        self.buf.ptr.as_ptr()
    }

    /// Copyies a contiguous memory block (len long) from src to dst
    unsafe fn copy(&mut self, src: usize, dst: usize, len: usize) {
        unsafe {
            ptr::copy(self.ptr().add(src), self.ptr().add(dst), len);
        }
    }

    /// Copyies a contiguous memory block (len long) from src to dst
    unsafe fn copy_nooverlap(&mut self, src: usize, dst: usize, len: usize) {
        unsafe {
            ptr::copy_nonoverlapping(self.ptr().add(src), self.ptr().add(dst), len);
        }
    }

    unsafe fn buffer_read(&mut self, off: usize) -> T {
        unsafe { ptr::read(self.ptr().add(off)) }
    }
    unsafe fn buffer_write(&mut self, off: usize, value: T) {
        unsafe {
            ptr::write(self.ptr().add(off), value);
        }
    }

    fn slice_range(&self) -> (Range<usize>, Range<usize>) {
        let Range { start, end } = (0..self.len);
        let len = self.len;
        if len == 0 {
            (0..0, 0..0)
        } else {
            let wrap_start = self.to_physical_idx(start);

            let head_len = self.cap() - wrap_start;

            if head_len >= len {
                (wrap_start..wrap_start + len, 0..0)
            } else {
                let tail_len = len - head_len;
                (wrap_start..self.cap(), 0..tail_len)
            }
        }
    }

    fn buffer_range(&self, range: Range<usize>) -> *mut [T] {
        unsafe {
            ptr::slice_from_raw_parts_mut(self.ptr().add(range.start), range.end - range.start)
        }
    }

    fn as_slice(&self) -> (&[T], &[T]) {
        let (r_a, r_b) = self.slice_range();

        unsafe { (&*self.buffer_range(r_a), &*self.buffer_range(r_b)) }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        let (a, b) = self.as_slice();
        Iter {
            i1: a.iter(),
            i2: b.iter(),
        }
    }

    fn to_vec_physical(&self) -> Vec<T> {
        let mut vec = Vec::new();
        unsafe {
            for idx in 0..self.cap() {
                vec.push(ptr::read(self.ptr().add(idx)))
            }
        }
        vec
    }

    pub fn push_front(&mut self, value: T) {
        if self.is_full() {
            self.grow();
        }

        self.head = self.wrap_sub(self.head, 1);
        self.len += 1;

        unsafe {
            self.buffer_write(self.head, value);
        }
    }

    pub fn push_back(&mut self, value: T) {
        if self.is_full() {
            self.grow();
        }

        unsafe { self.buffer_write(self.to_physical_idx(self.len), value) }
        self.len += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empth() {
            None
        } else {
            let old_head = self.head;
            self.head = self.to_physical_idx(1);
            self.len -= 1;
            Some(unsafe { self.buffer_read(old_head) })
        }
    }
    pub fn pop_back(&mut self) -> Option<T> {
        if self.is_empth() {
            None
        } else {
            self.len -= 1;
            Some(unsafe { self.buffer_read(self.to_physical_idx(self.len)) })
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Deque<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Deque")
            .field("head", &self.head)
            .field("len ", &self.len)
            .field("cap ", &self.cap())
            .finish();
        f.write_str("\nlogical\n");
        f.debug_list().entries(self.iter()).finish();
        f.write_str("\nphysical\n");
        f.debug_list().entries(self.to_vec_physical()).finish()
    }
}

pub struct IntoIter<T> {
    inner: Deque<T>,
}
impl<T> IntoIter<T> {
    fn new(inner: Deque<T>) -> Self {
        Self { inner }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.inner.len;
        (len, Some(len))
    }
}

impl<T> IntoIterator for Deque<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

pub struct Iter<'a, T: 'a> {
    i1: slice::Iter<'a, T>,
    i2: slice::Iter<'a, T>,
}
impl<'a, T> Iter<'a, T> {
    fn new(i1: slice::Iter<'a, T>, i2: slice::Iter<'a, T>) -> Self {
        Self { i1, i2 }
    }

    fn len(&self) -> usize {
        self.i1.len() + self.i2.len()
    }
}
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.i1.next() {
            Some(val) => Some(val),
            None => {
                mem::swap(&mut self.i1, &mut self.i2);
                self.i1.next()
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

#[test]
fn test_deque_push_front() {
    let mut dq = Deque::new();
    println!("{:?}", dq);
    for i in 1..10 {
        dq.push_front(i);
        println!("{:?}", dq);
    }

    for x in dq.iter() {
        println!("{x}")
    }
}

#[test]
fn test_deque_push_back() {
    let mut dq = Deque::new();
    println!("{:?}", dq);
    for i in 1..10 {
        dq.push_back(i);
        println!("{:?}", dq);
    }

    for x in dq.iter() {
        println!("{x}")
    }
}
