#![allow(dead_code)]

use std::{
    alloc,
    alloc::Layout,
    marker::PhantomData,
    mem,
    ops::Index,
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
};

use crate::iter::RawIter;

#[derive(Debug)]
struct RawVec<T> {
    ptr: NonNull<T>,
    cap: usize,
}
unsafe impl<T: Send> Send for RawVec<T> {}
unsafe impl<T: Sync> Sync for RawVec<T> {}

impl<T> RawVec<T> {
    fn new() -> Self {
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

    fn grow(&mut self) {
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

#[derive(Debug)]
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
    iter: RawIter<T>,
}

impl<T> Iterator for _IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T> DoubleEndedIterator for _IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
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
            let iter = RawIter::new(&self);
            let buf = ptr::read(&self.buf);
            mem::forget(self);

            _IntoIter { buf, iter }
        }
    }
}

pub struct _Drain<'a, T: 'a> {
    vec: PhantomData<&'a mut Vec<T>>,
    iter: RawIter<T>,
}

impl<'a, T> Iterator for _Drain<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for _Drain<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl<T> Vec<T> {
    pub fn drain(&mut self) -> _Drain<T> {
        unsafe {
            let iter = RawIter::new(&self);
            // preventing reading into freed memory
            self.len = 0;

            _Drain {
                iter,
                vec: PhantomData,
            }
        }
    }
}

#[test]
fn vec_test_push1() {
    println!(">>Test Start vec_test_push1");
    let mut vec = Vec::new();
    println!("Vec>> {:?}", vec);
    for i in 0..10000 {
        vec.push(i)
    }
    println!("Vec>> {:?}", vec);
    for i in vec.into_iter() {
        if i % 10 == 9 {
            println!("{}", i);
        } else {
            print!("{} ", i);
        }
    }
    println!(">>Test End vec_test_push1");
}

#[test]
fn vec_test_drain() {
    println!(">>Test Start vec_test_drain");
    let mut vec = Vec::new();
    println!("Vec>> {:?}", vec);
    for i in 0..10000 {
        vec.push(i)
    }
    println!("Vec>> {:?}", vec);
    for i in vec.drain() {
        if i % 10 == 9 {
            println!("{}", i);
        } else {
            print!("{} ", i);
        }
    }
    println!(">>Test End vec_test_drain");
}

#[test]
fn vec_test_reverse() {
    println!(">>Test Start vec_test_reverse");
    let mut vec = Vec::new();
    println!("Vec>> {:?}", vec);
    for i in 0..10000 {
        vec.push(i)
    }
    println!("Vec>> {:?}", vec);
    for i in vec.into_iter().rev() {
        if i % 10 == 9 {
            println!("{}", i);
        } else {
            print!("{} ", i);
        }
    }
    println!(">>Test End vec_test_reverse");
}

#[test]
fn vec_test_index() {
    println!(">>Test Start vec_test_index");
    let mut vec = Vec::new();
    println!("Vec>> {:?}", vec);
    for i in 0..10000 {
        vec.push(i)
    }
    println!("Vec>> {:?}", vec);
    for i in 0..10000 {
        let t = vec[i];
        if t % 10 == 9 {
            println!("{}", t);
        } else {
            print!("{} ", t);
        }
    }
    println!(">>Test End vec_test_index");
}

#[test]
fn vec_test_zst() {
    println!(">>Test Start vec_test_zst");
    let mut vec = Vec::new();

    #[derive(Debug, Copy, Clone)]
    struct ZeroSized;

    println!("Vec>> {:?}", vec);
    for _ in 0..10000 {
        vec.push(ZeroSized)
    }
    println!("Vec>> {:?}", vec);

    for i in 0..10000 {
        let t = vec[i];
        if i % 10 == 9 {
            println!("{:?}", t);
        } else {
            print!("{:?} ", t);
        }
    }
    println!(">>Test End vec_test_zst");
}
