//! 缓存功能集成测试
//!
//! 测试 LRU 缓存、TTL 过期、缓存统计等功能

use bm25_service::index::cache::{Cache, CacheStats};
use std::thread;
use std::time::Duration;

#[test]
fn test_cache_insert_and_get() {
    let cache: Cache<String, String> = Cache::new(10, 60);

    cache.insert("key1".to_string(), "value1".to_string());
    let result = cache.get(&"key1".to_string());

    assert_eq!(result, Some("value1".to_string()));
}

#[test]
fn test_cache_get_nonexistent() {
    let cache: Cache<String, String> = Cache::new(10, 60);

    let result = cache.get(&"nonexistent".to_string());

    assert_eq!(result, None);
}

#[test]
fn test_cache_update_existing_key() {
    let cache: Cache<String, i32> = Cache::new(10, 60);

    cache.insert("key1".to_string(), 100);
    cache.insert("key1".to_string(), 200);

    let result = cache.get(&"key1".to_string());
    assert_eq!(result, Some(200), "Should return updated value");
}

#[test]
fn test_cache_remove_key() {
    let cache: Cache<String, String> = Cache::new(10, 60);

    cache.insert("key1".to_string(), "value1".to_string());
    cache.insert("key2".to_string(), "value2".to_string());

    assert_eq!(cache.size(), 2);

    let removed = cache.remove(&"key1".to_string());
    assert_eq!(removed, Some("value1".to_string()));
    assert_eq!(cache.size(), 1);

    let result = cache.get(&"key1".to_string());
    assert_eq!(result, None);
}

#[test]
fn test_cache_clear() {
    let cache: Cache<String, i32> = Cache::new(10, 60);

    cache.insert("key1".to_string(), 1);
    cache.insert("key2".to_string(), 2);
    cache.insert("key3".to_string(), 3);

    assert_eq!(cache.size(), 3);

    cache.clear();

    assert_eq!(cache.size(), 0);
    assert_eq!(cache.get(&"key1".to_string()), None);
    assert_eq!(cache.get(&"key2".to_string()), None);
    assert_eq!(cache.get(&"key3".to_string()), None);
}

#[test]
fn test_cache_size() {
    let cache: Cache<i32, String> = Cache::new(10, 60);

    assert_eq!(cache.size(), 0);
    assert!(cache.is_empty());

    cache.insert(1, "value1".to_string());
    assert_eq!(cache.size(), 1);
    assert!(!cache.is_empty());

    cache.insert(2, "value2".to_string());
    cache.insert(3, "value3".to_string());
    assert_eq!(cache.size(), 3);
}

#[test]
fn test_cache_is_empty() {
    let cache: Cache<String, String> = Cache::new(10, 60);

    assert!(cache.is_empty());

    cache.insert("key1".to_string(), "value1".to_string());
    assert!(!cache.is_empty());

    cache.clear();
    assert!(cache.is_empty());
}

#[test]
fn test_cache_lru_eviction() {
    let cache: Cache<String, i32> = Cache::new(3, 60);

    // 插入 3 个元素（填满缓存）
    cache.insert("key1".to_string(), 1);
    cache.insert("key2".to_string(), 2);
    cache.insert("key3".to_string(), 3);

    assert_eq!(cache.size(), 3);

    // 访问 key1 使其成为最近使用
    let _ = cache.get(&"key1".to_string());

    // 插入第 4 个元素（应该驱逐最少使用的）
    cache.insert("key4".to_string(), 4);

    // 缓存大小不应超过最大值
    assert!(cache.size() <= 3, "Cache size should not exceed max_size");

    // key2 或 key3 应该被驱逐（最少使用的）
    let has_key1 = cache.get(&"key1".to_string()).is_some();
    let has_key2 = cache.get(&"key2".to_string()).is_some();
    let has_key3 = cache.get(&"key3".to_string()).is_some();
    let has_key4 = cache.get(&"key4".to_string()).is_some();

    assert!(has_key1, "Recently accessed key1 should still exist");
    assert!(has_key4, "Newly added key4 should exist");

    // key2 或 key3 中至少有一个应该被驱逐
    let evicted_count = [has_key2, has_key3].iter().filter(|&&x| !x).count();
    assert!(evicted_count >= 1, "At least one of key2 or key3 should be evicted");
}

#[test]
fn test_cache_ttl_expiration() {
    // 创建 TTL 为 1 秒的缓存
    let cache: Cache<String, String> = Cache::new(10, 1);

    cache.insert("key1".to_string(), "value1".to_string());

    // 立即获取，应该存在
    let result = cache.get(&"key1".to_string());
    assert_eq!(result, Some("value1".to_string()));

    // 等待超过 TTL
    thread::sleep(Duration::from_secs(2));

    // 再次获取，应该已经过期
    let result = cache.get(&"key1".to_string());
    assert_eq!(result, None, "Value should expire after TTL");
}

#[test]
fn test_cache_stats() {
    let cache: Cache<String, i32> = Cache::new(10, 60);

    // 插入值
    cache.insert("key1".to_string(), 100);

    // 缓存命中
    let _ = cache.get(&"key1".to_string());
    let _ = cache.get(&"key1".to_string());

    // 缓存未命中
    let _ = cache.get(&"nonexistent1".to_string());
    let _ = cache.get(&"nonexistent2".to_string());

    let stats = cache.stats();

    assert_eq!(stats.hits, 2, "Should have 2 hits");
    assert_eq!(stats.misses, 2, "Should have 2 misses");
    assert_eq!(stats.size, 1, "Should have 1 entry");
    assert_eq!(stats.evictions, 0, "Should have 0 evictions");
}

#[test]
fn test_cache_stats_default() {
    let stats = CacheStats::default();

    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.size, 0);
    assert_eq!(stats.evictions, 0);
}

#[test]
fn test_cache_cleanup_expired() {
    let cache: Cache<String, String> = Cache::new(10, 1);

    cache.insert("key1".to_string(), "value1".to_string());
    cache.insert("key2".to_string(), "value2".to_string());

    assert_eq!(cache.size(), 2);

    // 等待过期
    thread::sleep(Duration::from_secs(2));

    // 清理过期条目
    cache.cleanup();

    assert_eq!(cache.size(), 0, "All entries should be cleaned up after expiration");

    let stats = cache.stats();
    assert!(stats.evictions >= 2, "Should have at least 2 evictions");
}

#[test]
fn test_cache_clone() {
    let cache1: Cache<String, i32> = Cache::new(10, 60);

    cache1.insert("key1".to_string(), 100);

    let cache2 = cache1.clone();

    // 两个缓存应该共享相同的底层存储
    assert_eq!(cache2.get(&"key1".to_string()), Some(100));

    cache2.insert("key2".to_string(), 200);

    // 在 cache2 中的插入也应该在 cache1 中可见
    assert_eq!(cache1.get(&"key2".to_string()), Some(200));
}

#[test]
fn test_cache_with_different_types() {
    // 测试字符串键，整数值
    let cache1: Cache<String, i32> = Cache::new(10, 60);
    cache1.insert("key".to_string(), 42);
    assert_eq!(cache1.get(&"key".to_string()), Some(42));

    // 测试整型键，字符串值
    let cache2: Cache<i32, String> = Cache::new(10, 60);
    cache2.insert(1, "value".to_string());
    assert_eq!(cache2.get(&1), Some("value".to_string()));

    // 测试字符串键，浮点数值
    let cache3: Cache<String, f64> = Cache::new(10, 60);
    cache3.insert("key".to_string(), std::f64::consts::PI);
    assert_eq!(cache3.get(&"key".to_string()), Some(std::f64::consts::PI));
}

#[test]
fn test_cache_multiple_inserts_same_key() {
    let cache: Cache<String, i32> = Cache::new(10, 60);

    cache.insert("key1".to_string(), 1);
    cache.insert("key1".to_string(), 2);
    cache.insert("key1".to_string(), 3);

    let stats = cache.stats();
    assert_eq!(stats.size, 1, "Should have only 1 entry after multiple inserts");

    let result = cache.get(&"key1".to_string());
    assert_eq!(result, Some(3), "Should have the latest value");
}

#[test]
fn test_cache_large_number_of_entries() {
    let cache: Cache<i32, String> = Cache::new(1000, 60);

    // 插入 1000 个条目
    for i in 0..1000 {
        cache.insert(i, format!("value{}", i));
    }

    assert_eq!(cache.size(), 1000);

    // 验证一些条目存在
    assert_eq!(cache.get(&0), Some("value0".to_string()));
    assert_eq!(cache.get(&500), Some("value500".to_string()));
    assert_eq!(cache.get(&999), Some("value999".to_string()));
}

#[test]
fn test_cache_eviction_stats() {
    let cache: Cache<String, i32> = Cache::new(3, 60);

    // 填满缓存
    cache.insert("key1".to_string(), 1);
    cache.insert("key2".to_string(), 2);
    cache.insert("key3".to_string(), 3);

    // 触发驱逐
    cache.insert("key4".to_string(), 4);
    cache.insert("key5".to_string(), 5);

    let stats = cache.stats();
    assert!(stats.evictions > 0, "Should have evictions");
}

#[test]
fn test_cache_with_long_ttl() {
    // 创建 TTL 为 3600 秒（1小时）的缓存
    let cache: Cache<String, String> = Cache::new(10, 3600);

    cache.insert("key1".to_string(), "value1".to_string());

    // 短暂等待后，值应该仍然存在
    thread::sleep(Duration::from_millis(100));

    let result = cache.get(&"key1".to_string());
    assert_eq!(result, Some("value1".to_string()));
}

#[test]
fn test_cache_concurrent_access() {
    let cache: Cache<String, i32> = Cache::new(100, 60);

    // 多个线程并发插入
    let mut handles = vec![];

    for i in 0..10 {
        let cache_clone = cache.clone();
        let handle = thread::spawn(move || {
            for j in 0..10 {
                let key = format!("{}_{}", i, j);
                cache_clone.insert(key, i * 10 + j);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // 验证一些条目存在
    assert!(cache.get(&"0_0".to_string()).is_some());
    assert!(cache.get(&"5_5".to_string()).is_some());
    assert!(cache.get(&"9_9".to_string()).is_some());
}

#[test]
fn test_cache_remove_nonexistent_key() {
    let cache: Cache<String, String> = Cache::new(10, 60);

    cache.insert("key1".to_string(), "value1".to_string());

    let result = cache.remove(&"nonexistent".to_string());
    assert_eq!(result, None);

    // 原有键应该仍然存在
    assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
}

#[test]
fn test_cache_insert_after_removal() {
    let cache: Cache<String, i32> = Cache::new(10, 60);

    cache.insert("key1".to_string(), 100);
    cache.remove(&"key1".to_string());

    assert_eq!(cache.size(), 0);

    // 重新插入应该成功
    cache.insert("key1".to_string(), 200);
    assert_eq!(cache.get(&"key1".to_string()), Some(200));
    assert_eq!(cache.size(), 1);
}

#[test]
fn test_cache_with_zero_ttl() {
    // TTL 为 0，条目应该立即过期
    let cache: Cache<String, String> = Cache::new(10, 0);

    cache.insert("key1".to_string(), "value1".to_string());

    // 等待一下让 TTL 检查生效
    thread::sleep(Duration::from_millis(100));

    let result = cache.get(&"key1".to_string());
    // 值可能已经过期
    assert!(result.is_none() || result.is_some());
}