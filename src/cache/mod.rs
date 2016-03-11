use std::hash::Hash;
use std::iter::Iterator;

use linked_hash_map::{Iter, LinkedHashMap};


pub struct LruCache<K, V> {
    inner: LinkedHashMap<K, WriteBack<V>>,
    cap: usize,
}

pub struct WriteBack<V> {
    inner: V,
    dirty: bool,
}

impl<V> WriteBack<V> {
    pub fn new(v: V) -> WriteBack<V> {
        WriteBack { inner: v, dirty: false }
    }

    pub fn into_inner(self) -> V {
        self.inner
    }
}

impl<K: Eq + Hash, V> LruCache<K, V> {
    pub fn with_capacity(cap: usize) -> LruCache<K, V> {
        LruCache {
            inner: LinkedHashMap::with_capacity(cap),
            cap: cap,
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn insert(&mut self, k: K, mut v: WriteBack<V>) -> Option<(K, WriteBack<V>)> {
        if self.inner.get(&k).is_some() {
            v.dirty = true;
        }
        self.inner.insert(k, v);
        if self.len() >= self.cap {
            self.inner.pop_front()
        } else {
            None
        }
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        self.inner.get(k)
    }

    pub fn remove(&mut self, k: &K) -> Option<WriteBack<V>> {
        self.inner.remove(k)
    }

    pub fn refresh(&mut self, k: &K) {
        self.inner.get_refresh(&k);
    }

    pub fn resize(&mut self, cap: usize) -> bool {
        // TODO should this allow any new size and just write everything back if smaller? This
        // probably doesn't matter.
        if cap > self.cap {
            self.inner.reserve(cap - self.cap);
            true
        } else {
            false
        }
    }

    pub fn flush(&mut self) -> Iter<K, WriteBack<V>> {
        self.inner.into_iter()
    }
}

