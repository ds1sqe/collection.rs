use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

const HASHMAP_ARRAY_MAX_SIZE: usize = 64;

pub struct HashMap<K, V> {
    len: usize,
    arr: [Option<HashNode<K, V>>; HASHMAP_ARRAY_MAX_SIZE],
}
pub struct HashNode<K, V> {
    key: K,
    value: V,
    next: Option<Box<HashNode<K, V>>>,
}

fn __hash_key<K: Hash>(key: K) -> usize {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    let hash = hasher.finish();
    hash as usize % HASHMAP_ARRAY_MAX_SIZE
}

impl<K, V> HashMap<K, V>
where
    K: Hash + std::cmp::PartialEq + Clone,
    V: Clone,
{
    const INIT: Option<HashNode<K, V>> = None;
    pub fn new() -> Self {
        HashMap {
            len: 0,
            arr: [Self::INIT; HASHMAP_ARRAY_MAX_SIZE],
        }
    }

    fn insert_new_node(&mut self, key: K, value: V, idx: usize) -> Option<V> {
        let new_node = HashNode::new(key, value);

        self.arr[idx] = Some(new_node);
        self.len += 1;
        None
    }

    fn update_node(&mut self, key: K, value: V, idx: usize) -> Option<V> {
        let old_node = self.arr[idx].as_mut().unwrap();
        // if have duplicated key, update with given new value
        if old_node.key == key {
            let old_val = old_node.value.clone();
            old_node.value = value;
            return Some(old_val);
        }

        let mut cur_node = old_node;

        while cur_node.next.is_some() {
            let next_node = cur_node.next.as_mut().unwrap();

            if next_node.key == key {
                let old_val = cur_node.value.clone();
                next_node.value = value;
                return Some(old_val);
            }

            cur_node = next_node;
        }

        let new_node = HashNode::new(key, value);
        cur_node.next = Some(Box::new(new_node));
        self.len += 1;

        None
    }

    pub fn put(&mut self, key: K, value: V) -> Option<V> {
        let idx = __hash_key(key.clone());

        match &self.arr[idx] {
            Some(_) => self.update_node(key, value, idx),
            None => self.insert_new_node(key, value, idx),
        }
    }

    fn search(&self, key: K, idx: usize) -> Option<V> {
        let mut cur_node = self.arr[idx].as_ref().unwrap();

        if cur_node.key == key {
            return Some(cur_node.value.clone());
        }

        while let Some(next_node) = cur_node.next.as_ref() {
            if next_node.key == key {
                return Some(next_node.value.clone());
            }
            cur_node = next_node;
        }
        None
    }

    pub fn get(&self, key: K) -> Option<V> {
        let idx = __hash_key(key.clone());

        match &self.arr[idx] {
            Some(_) => self.search(key, idx),
            None => None,
        }
    }

    // pub fn remove(&self, key: K) -> Option<V> {}
}

impl<K, V> HashNode<K, V> {
    pub fn new(key: K, value: V) -> HashNode<K, V> {
        HashNode {
            key,
            value,
            next: None,
        }
    }
}

#[test]
fn hashmap_test_search() {
    let mut hm = HashMap::new();
    for i in 0..100 {
        hm.put(i, (i * 10 + 1).to_string());
    }

    for i in 0..100 {
        println!("(k,v):({i},{})", hm.get(i).unwrap());
    }
}
