#![allow(dead_code)]

use std::{
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    ptr::{self},
};

use super::raw::raw_iter::RawIter;
use super::raw::raw_vec::RawVec;

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
