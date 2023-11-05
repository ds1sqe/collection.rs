#![allow(unused)]
use std::{
    cmp,
    collections::BinaryHeap,
    mem::{swap, ManuallyDrop},
    ptr,
};

use super::vec::Vec;

#[derive(Debug)]
struct Heap<T> {
    data: Vec<T>,
}

impl<T> Heap<T>
where
    T: PartialOrd,
    T: Copy,
{
    pub fn peek(&self) -> Option<T> {
        if !self.is_empty() {
            Some(self.data[0])
        } else {
            return None;
        }
    }
}

impl<T> Heap<T>
where
    T: PartialOrd,
{
    pub fn new() -> Self {
        Heap { data: Vec::new() }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Heap {
            data: Vec::with_capacity(cap),
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.len() == 0
    }

    pub fn push(&mut self, value: T) {
        let old_len = self.data.len();

        self.data.push(value);

        self.sift_up(0, old_len);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.data.pop().map(|mut el| {
            if !self.is_empty() {
                swap(&mut el, &mut self.data[0]);
                self.sift_down(0);
            }
            el
        })
    }

    fn sift_up(&mut self, start: usize, idx: usize) -> usize {
        let mut hole = unsafe { Hole::new(&mut self.data, idx) };

        while hole.idx() > start {
            let parent = (hole.idx - 1) / 2;

            if hole.el() <= hole.get(parent) {
                break;
            }

            hole.move_to(parent)
        }

        hole.idx()
    }

    fn sift_down(&mut self, mut idx: usize) {
        let end = self.data.len();
        let start = idx;

        let mut hole = unsafe { Hole::new(&mut self.data, idx) };
        let mut child = 2 * hole.idx() + 1;

        while child <= end.saturating_sub(2) {
            child += (hole.get(child) <= hole.get(child + 1)) as usize;

            hole.move_to(child);

            child = 2 * hole.idx() + 1;
        }

        if child == end - 1 {
            hole.move_to(child);
        }

        idx = hole.idx();
        drop(hole);

        self.sift_up(start, idx);
    }
}

/// Hole represent index without valid value
struct Hole<'a, T: 'a> {
    data: &'a mut [T],
    el: ManuallyDrop<T>,
    idx: usize,
}

impl<'a, T> Hole<'a, T> {
    fn new(data: &'a mut [T], idx: usize) -> Self {
        assert!(idx < data.len());
        let el = unsafe { ptr::read(data.get_unchecked(idx)) };
        Hole {
            data,
            el: ManuallyDrop::new(el),
            idx,
        }
    }

    /// Returns a refference to removed element
    fn el(&self) -> &T {
        &self.el
    }

    fn idx(&self) -> usize {
        self.idx
    }
    /// Returns a refference to element at idx
    fn get(&self, idx: usize) -> &T {
        assert!(idx < self.data.len());
        unsafe { self.data.get_unchecked(idx) }
    }

    /// move hole to new idx
    /// move given idx's data to hole's one
    /// and change hole's idx to given idx
    fn move_to(&mut self, idx: usize) {
        unsafe {
            let ptr = self.data.as_mut_ptr();
            let idx_ptr = ptr.add(idx);
            let cur_ptr = ptr.add(self.idx);
            ptr::copy_nonoverlapping(idx_ptr, cur_ptr, 1);
        }
        self.idx = idx;
    }
}

impl<T> Drop for Hole<'_, T> {
    /// move self.el to idx. fill hole
    fn drop(&mut self) {
        unsafe {
            let idx = self.idx;
            ptr::copy_nonoverlapping(&*self.el, self.data.get_unchecked_mut(idx), 1);
        }
    }
}

#[test]
fn heap_test_new_push() {
    let mut hp = Heap::new();
    hp.push(3);
    hp.push(4);
    hp.push(1);
    hp.push(2);
    hp.push(5);

    println!("{:?}", hp);

    for _ in 0..5 {
        println!("{:?}", hp.pop());

        println!("{:?}", hp);
    }
}
