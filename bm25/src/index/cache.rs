use std::collections::HashMap;
use std::time::{Instant, Duration};

#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub evictions: usize,
    pub size: usize,
}

#[derive(Clone)]
struct CacheEntry<V> {
    value: V,
    last_accessed: Instant,
}

#[derive(Clone)]
pub struct Cache<K, V> {
    entries: HashMap<K, CacheEntry<V>>,
    access_order: Vec<K>,
    max_size: usize,
    ttl_seconds: u64,
    stats: CacheStats,
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> Cache<K, V> {
    pub fn new(max_size: usize, ttl_seconds: u64) -> Self {
        Cache {
            entries: HashMap::new(),
            access_order: Vec::new(),
            max_size,
            ttl_seconds,
            stats: CacheStats::default(),
        }
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(entry) = self.entries.get_mut(key) {
            let now = Instant::now();
            let age = now.duration_since(entry.last_accessed);

            if age < Duration::from_secs(self.ttl_seconds) {
                entry.last_accessed = now;
                self.stats.hits += 1;

                // Update access order (move to end = most recently used)
                if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                    self.access_order.remove(pos);
                    self.access_order.push(key.clone());
                }

                Some(entry.value.clone())
            } else {
                // Entry expired
                self.entries.remove(key);
                if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                    self.access_order.remove(pos);
                }
                self.stats.size = self.entries.len();
                self.stats.misses += 1;
                None
            }
        } else {
            self.stats.misses += 1;
            None
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let now = Instant::now();

        // If key already exists, update it
        if self.entries.contains_key(&key) {
            self.entries.insert(key.clone(), CacheEntry {
                value,
                last_accessed: now,
            });

            // Update access order
            if let Some(pos) = self.access_order.iter().position(|k| k == &key) {
                self.access_order.remove(pos);
            }
            self.access_order.push(key);
            return;
        }

        // Evict if necessary
        while self.entries.len() >= self.max_size {
            if let Some(lru_key) = self.access_order.first() {
                let key_to_remove = lru_key.clone();
                self.entries.remove(&key_to_remove);
                self.access_order.remove(0);
                self.stats.evictions += 1;
            } else {
                break;
            }
        }

        // Insert new entry
        self.entries.insert(key.clone(), CacheEntry {
            value,
            last_accessed: now,
        });
        self.access_order.push(key);
        self.stats.size = self.entries.len();
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(entry) = self.entries.remove(key) {
            if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                self.access_order.remove(pos);
            }
            self.stats.size = self.entries.len();
            Some(entry.value)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
        self.stats.size = 0;
    }

    pub fn cleanup(&mut self) {
        let now = Instant::now();
        let ttl_duration = Duration::from_secs(self.ttl_seconds);

        // Collect expired keys
        let expired_keys: Vec<K> = self.entries
            .iter()
            .filter(|(_, entry)| now.duration_since(entry.last_accessed) >= ttl_duration)
            .map(|(key, _)| key.clone())
            .collect();

        // Remove expired entries
        for key in expired_keys {
            self.entries.remove(&key);
            if let Some(pos) = self.access_order.iter().position(|k| k == &key) {
                self.access_order.remove(pos);
            }
            self.stats.evictions += 1;
        }

        self.stats.size = self.entries.len();
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.stats.hits,
            misses: self.stats.misses,
            evictions: self.stats.evictions,
            size: self.stats.size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic_operations() {
        let mut cache: Cache<String, i32> = Cache::new(10, 60);

        assert_eq!(cache.get(&"key1".to_string()), None);
        assert_eq!(cache.size(), 0);

        cache.insert("key1".to_string(), 42);
        assert_eq!(cache.get(&"key1".to_string()), Some(42));
        assert_eq!(cache.size(), 1);

        cache.insert("key1".to_string(), 100);
        assert_eq!(cache.get(&"key1".to_string()), Some(100));

        cache.remove(&"key1".to_string());
        assert_eq!(cache.get(&"key1".to_string()), None);
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_cache_lru_eviction() {
        let mut cache: Cache<String, i32> = Cache::new(3, 60);

        cache.insert("key1".to_string(), 1);
        cache.insert("key2".to_string(), 2);
        cache.insert("key3".to_string(), 3);

        assert_eq!(cache.size(), 3);

        cache.insert("key4".to_string(), 4);

        assert!(cache.size() <= 3);
    }

    #[test]
    fn test_cache_stats() {
        let mut cache: Cache<String, i32> = Cache::new(10, 60);

        cache.insert("key1".to_string(), 1);
        cache.get(&"key1".to_string());
        cache.get(&"nonexistent".to_string());

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.size, 1);
    }

    #[test]
    fn test_cache_clear() {
        let mut cache: Cache<String, i32> = Cache::new(10, 60);

        cache.insert("key1".to_string(), 1);
        cache.insert("key2".to_string(), 2);

        assert_eq!(cache.size(), 2);

        cache.clear();

        assert_eq!(cache.size(), 0);
        assert_eq!(cache.get(&"key1".to_string()), None);
    }
}
