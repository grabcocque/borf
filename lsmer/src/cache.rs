use std::collections::HashMap;

/// A simple in-memory cache.
pub struct Cache<K, V> {
    // TODO: add internal fields
    _map: HashMap<K, V>,
}

impl<K: std::hash::Hash + Eq, V> Cache<K, V> {
    /// Create a new cache with the given capacity.
    pub fn new(_capacity: usize) -> Self {
        Cache {
            _map: HashMap::new(),
        }
    }

    /// Get a reference to a value by key.
    pub fn get(&self, key: &K) -> Option<&V> {
        self._map.get(key)
    }

    /// Insert a key-value pair into the cache.
    pub fn put(&mut self, key: K, value: V) {
        self._map.insert(key, value);
    }
}
