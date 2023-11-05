use std::{
    collections::hash_map::DefaultHasher,
    fmt,
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

    fn search_and_remove(&mut self, key: K, idx: usize) -> Option<V> {
        let mut cur_node = self.arr[idx].as_mut().unwrap();

        if cur_node.key == key {
            let value = cur_node.value.clone();

            if let Some(next_node) = cur_node.next.take() {
                self.arr[idx] = Some(*next_node);
            } else {
                self.arr[idx] = None;
            }

            self.len -= 1;
            return Some(value);
        } else {
            while cur_node.next.is_some() {
                let next = cur_node.next.as_mut().unwrap();
                if next.key == key {
                    let value = next.value.clone();

                    if let Some(next_of_next) = next.next.take() {
                        cur_node.next = Some(next_of_next);
                    } else {
                        cur_node.next = None;
                    }

                    self.len -= 1;
                    return Some(value);
                } else {
                    cur_node = cur_node.next.as_mut().unwrap();
                }
            }
            return None;
        }
    }

    pub fn remove(&mut self, key: K) -> Option<V> {
        let idx = __hash_key(key.clone());
        match &self.arr[idx] {
            Some(_) => self.search_and_remove(key, idx),
            None => None,
        }
    }
}

impl<K, V> fmt::Debug for HashMap<K, V>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HashMap entities:{}\n", &self.len).unwrap();

        for (pos, node) in self.arr.iter().enumerate() {
            if node.is_none() {
                write!(f, "[arr@{pos}]: No Exists\n").unwrap();
            } else {
                write!(f, "[arr@{pos}]:\n").unwrap();
                let node = node.as_ref().unwrap();
                write!(f, "{:?}:{:?}\n", node.key, node.value).unwrap();

                let mut padding = 1;
                let mut cur_node = node.next.as_ref();
                while cur_node.is_some() {
                    for _ in 0..padding {
                        write!(f, ">").unwrap();
                    }
                    write!(
                        f,
                        "{:?}:{:?}\n",
                        cur_node.unwrap().key,
                        cur_node.unwrap().value
                    )
                    .unwrap();
                    cur_node = cur_node.unwrap().next.as_ref();
                    padding += 1;
                }
            }
        }
        Ok(())
    }
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

impl<K, V> fmt::Debug for HashNode<K, V>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HashNode")
            .field("key", &self.key)
            .field("value", &self.value)
            .field("next", &self.next)
            .finish()
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

#[test]
fn hashmap_test_remove() {
    let mut hm = HashMap::new();
    for i in 0..100 {
        hm.put(i, (i * 10 + 1).to_string());
    }

    for i in 0..100 {
        if i == 42 || i % 10 == 0 {
            hm.remove(i);
        }
    }

    for i in 0..100 {
        println!(
            "(k,v):({i},{})",
            hm.get(i).unwrap_or("No Exists".to_string())
        );
    }
}

#[test]
fn hashmap_test_debug() {
    let mut hm = HashMap::new();
    for i in 0..100 {
        hm.put(i, (i * 10 + 1).to_string());
    }

    for i in 0..100 {
        if i == 42 || i % 10 == 0 {
            hm.remove(i);
        }
    }

    println!("{:?}", hm);
}
