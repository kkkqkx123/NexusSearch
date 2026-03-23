# Inversearch 服务技术分析

## 概述

本文档详细分析从 JavaScript 实现到 Rust 实现的技术转换，包括数据结构映射、算法转换、性能优化等。

---

## 一、数据结构映射

### 1.1 倒排索引

#### JavaScript 实现

```javascript
// src/index.js
this.map = new Map();
// 结构: Map<Term, Array<ResolutionSlot>>
// ResolutionSlot: Array<DocID>
```

#### Rust 实现

```rust
use std::collections::HashMap;

pub struct Index {
    // 倒排索引: Term -> [ResolutionSlot]
    pub map: HashMap<String, Vec<Vec<DocId>>>,
    // 上下文索引: Keyword -> Term -> [ResolutionSlot]
    pub ctx: HashMap<String, HashMap<String, Vec<Vec<DocId>>>>,
    // 文档注册表: DocID -> [ResolutionSlot]
    pub reg: KeystoreSet<DocId>,
}
```

**转换要点**：

1. `Map` → `HashMap`：使用 Rust 的 HashMap 替代 JavaScript 的 Map
2. 嵌套数组结构保持一致：`Vec<Vec<DocId>>` 对应 `Array<ResolutionSlot>`
3. 使用泛型支持不同类型的文档 ID

### 1.2 密钥存储

#### JavaScript 实现

```javascript
// src/keystore.js
class KeystoreMap {
    constructor(bitlength = 8) {
        this.index = create_object();
        this.refs = [];
        this.size = 0;
        this.crc = bitlength > 32 ? lcg64 : lcg;
        this.bit = bitlength;
    }
}
```

#### Rust 实现

```rust
pub struct KeystoreMap<K, V> {
    // 分片索引: Address -> Map
    pub index: Vec<HashMap<K, V>>,
    // 引用列表
    pub refs: Vec<HashMap<K, V>>,
    pub size: usize,
    // 哈希函数
    pub crc: fn(&K) -> usize,
    pub bit: u32,
}
```

**转换要点**：

1. 使用 `Vec<HashMap>` 实现分片存储
2. 哈希函数使用函数指针或 trait 对象
3. 支持大容量存储（超过 2^31 个元素）

### 1.3 编码器配置

#### JavaScript 实现

```javascript
// src/encoder.js
export default function Encoder(options = {}) {
    this.normalize = options.normalize;
    this.split = options.split;
    this.prepare = options.prepare;
    this.finalize = options.finalize;
    this.filter = options.filter;
    this.dedupe = options.dedupe;
    this.matcher = options.matcher;
    this.mapper = options.mapper;
    this.stemmer = options.stemmer;
    this.replacer = options.replacer;
    this.minlength = options.minlength;
    this.maxlength = options.maxlength;
    this.rtl = options.rtl;
    this.cache = options.cache;
}
```

#### Rust 实现

```rust
pub struct Encoder {
    // 规范化
    pub normalize: NormalizeMode,
    // 分词
    pub split: Option<Regex>,
    // 预处理
    pub prepare: Option<Box<dyn Fn(&str) -> String>>,
    // 最终处理
    pub finalize: Option<Box<dyn Fn(&mut Vec<String>)>>,
    // 停用词
    pub filter: Option<HashSet<String>>,
    // 去重
    pub dedupe: bool,
    // 词形映射
    pub matcher: Option<HashMap<String, String>>,
    // 字符映射
    pub mapper: Option<HashMap<char, char>>,
    // 词干提取
    pub stemmer: Option<HashMap<String, String>>,
    // 正则替换
    pub replacer: Vec<(Regex, String)>,
    // 最小长度
    pub minlength: usize,
    // 最大长度
    pub maxlength: usize,
    // 从右到左
    pub rtl: bool,
    // 缓存
    pub cache: Option<Cache<String, Vec<String>>>,
}
```

**转换要点**：

1. 使用 `Option` 处理可选配置
2. 使用 `Box<dyn Fn>` 处理函数类型
3. 使用 `HashSet` 存储停用词
4. 使用 `Regex` 处理正则表达式

---

## 二、算法转换

### 2.1 评分算法

#### JavaScript 实现

```javascript
// src/index/add.js
function get_score(resolution, length, i, term_length, x) {
    return i && (resolution > 1) ? (
        (length + (term_length || 0)) <= resolution ?
            i + (x || 0)
        :
            ((resolution - 1) / (length + (term_length || 0)) * (i + (x || 0)) + 1) | 0
    ) : 0;
}
```

#### Rust 实现

```rust
fn get_score(resolution: usize, length: usize, i: usize, term_length: Option<usize>, x: Option<usize>) -> usize {
    if i == 0 || resolution <= 1 {
        return 0;
    }

    let total_length = length + term_length.unwrap_or(0);
    let offset = x.unwrap_or(0);

    if total_length <= resolution {
        i + offset
    } else {
        ((resolution - 1) * (i + offset) / total_length + 1) as usize
    }
}
```

**转换要点**：

1. 使用 `Option` 处理可选参数
2. 移除位运算符 `| 0`，Rust 的整数除法自动截断
3. 保持算法逻辑完全一致

### 2.2 交集算法

#### JavaScript 实现

```javascript
// src/intersect.js
export function intersect(arrays, resolution, limit, offset, suggest, boost, resolve) {
    const length = arrays.length;
    let result = [];
    let check = create_object();
    let count;

    for(let y = 0, ids, id, res_arr, tmp; y < resolution; y++){
        for(let x = 0; x < length; x++){
            res_arr = arrays[x];
            if(y < res_arr.length && (ids = res_arr[y])){
                for(let z = 0; z < ids.length; z++){
                    id = ids[z];
                    if((count = check[id])){
                        check[id]++;
                    } else{
                        count = 0;
                        check[id] = 1;
                    }
                    tmp = result[count] || (result[count] = []);
                    tmp.push(id);
                }
            }
        }
    }

    return result[result.length - 1] || [];
}
```

#### Rust 实现

```rust
pub fn intersect(
    arrays: &[IntermediateSearchResults],
    resolution: usize,
    limit: usize,
    offset: usize,
) -> SearchResults {
    let length = arrays.len();
    let mut result: Vec<Vec<DocId>> = Vec::new();
    let mut check: HashMap<DocId, usize> = HashMap::new();

    for y in 0..resolution {
        for x in 0..length {
            if let Some(res_arr) = arrays.get(x) {
                if let Some(ids) = res_arr.get(y) {
                    for &id in ids {
                        let count = check.entry(id).or_insert(0);
                        *count += 1;
                        let slot = result.get_mut(*count).unwrap();
                        slot.push(id);
                    }
                }
            }
        }
    }

    result.get(length).cloned().unwrap_or_default()
}
```

**转换要点**：

1. 使用 `HashMap` 替代 `create_object()`
2. 使用迭代器遍历数组
3. 使用 `get_mut()` 获取可变引用
4. 使用 `cloned()` 克隆结果

### 2.3 哈希算法

#### JavaScript 实现

```javascript
// src/keystore.js
function lcg(str) {
    let range = 2 ** this.bit - 1;
    let crc = 0, bit = this.bit + 1;
    for(let i = 0; i < str.length; i++) {
        crc = (crc * bit ^ str.charCodeAt(i)) & range;
    }
    return this.bit === 32 ? crc + 2 ** 31 : crc;
}
```

#### Rust 实现

```rust
fn lcg(str: &str, bit: u32) -> u32 {
    let range = (1 << bit) - 1;
    let mut crc: u32 = 0;
    let bit = bit + 1;

    for byte in str.bytes() {
        crc = (crc.wrapping_mul(bit) ^ byte as u32) & range;
    }

    if bit == 32 {
        crc + (1 << 31)
    } else {
        crc
    }
}
```

**转换要点**：

1. 使用 `wrapping_mul()` 处理乘法溢出
2. 使用 `bytes()` 迭代字节
3. 使用位运算替代幂运算

---

## 三、性能优化

### 3.1 内存优化

#### JavaScript 实现

```javascript
// src/keystore.js
class KeystoreArray {
    constructor(arr) {
        this.index = arr ? [arr] : [];
        this.length = arr ? arr.length : 0;
    }
}
```

#### Rust 实现

```rust
pub struct KeystoreArray<T> {
    // 分片存储
    pub index: Vec<Vec<T>>,
    pub length: usize,
}

impl<T> KeystoreArray<T> {
    pub fn push(&mut self, value: T) {
        let last = self.index.last_mut().unwrap();
        if last.len() >= 2_i32.pow(31) as usize {
            self.index.push(Vec::new());
        }
        self.index.last_mut().unwrap().push(value);
        self.length += 1;
    }
}
```

**优化要点**：

1. 使用分片存储避免单个数组过大
2. 自动分片，每个分片最大 2^31 个元素
3. 使用 `Vec` 替代数组，动态扩容

### 3.2 查询优化

#### JavaScript 实现

```javascript
// src/index/search.js
Index.prototype.search = function(query, limit, options) {
    // 快速路径：单词语查询
    if(length === 1){
        return single_term_query.call(this, query_terms[0], "", limit, offset, resolve, enrich, tag);
    }

    // 快速路径：单上下文查询
    if(length === 2 && context && !suggest){
        return single_term_query.call(this, query_terms[1], query_terms[0], limit, offset, resolve, enrich, tag);
    }

    // 多词语查询
    for(let arr, term; index < length; index++){
        // ...
    }
}
```

#### Rust 实现

```rust
impl Index {
    pub fn search(&self, query: &str, options: &SearchOptions) -> SearchResults {
        let query_terms = self.encoder.encode(query, !self.depth);
        let length = query_terms.len();

        // 快速路径：单词语查询
        if length == 1 {
            return self.single_term_query(&query_terms[0], "", options);
        }

        // 快速路径：单上下文查询
        if length == 2 && self.depth > 0 && !options.suggest {
            return self.single_term_query(&query_terms[1], &query_terms[0], options);
        }

        // 多词语查询
        self.multi_term_query(&query_terms, options)
    }
}
```

**优化要点**：

1. 保持快速路径优化
2. 使用模式匹配替代条件判断
3. 减少不必要的克隆

### 3.3 缓存优化

#### JavaScript 实现

```javascript
// src/cache.js
export default function CacheClass(limit) {
    this.limit = (!limit || limit === true) ? 1000 : limit;
    this.cache = new Map();
    this.last = "";
}

CacheClass.prototype.set = function(key, value){
    this.cache.set(this.last = key, value);
    if(this.cache.size > this.limit){
        this.cache.delete(this.cache.keys().next().value);
    }
};
```

#### Rust 实现

```rust
use std::collections::HashMap;
use std::collections::linked_hash_map::LinkedHashMap;

pub struct Cache<K, V> {
    pub limit: usize,
    pub cache: LinkedHashMap<K, V>,
}

impl<K: Eq + Hash + Clone, V: Clone> Cache<K, V> {
    pub fn set(&mut self, key: K, value: V) {
        self.cache.insert(key.clone(), value);
        self.cache.to_back(&key);

        if self.cache.len() > self.limit {
            if let Some((oldest, _)) = self.cache.pop_front() {
                // 删除最旧的条目
            }
        }
    }
}
```

**优化要点**：

1. 使用 `LinkedHashMap` 实现 LRU
2. 使用 `to_back()` 将访问的条目移到末尾
3. 使用 `pop_front()` 删除最旧的条目

---

## 四、错误处理

### 4.1 JavaScript 错误处理

```javascript
// src/index.js
Index.prototype.add = function(id, content, _append, _skip_update){
    if(content && (id || (id === 0))){
        // 处理逻辑
    }
    return this;
};
```

### 4.2 Rust 错误处理

```rust
impl Index {
    pub fn add(&mut self, id: DocId, content: &str) -> Result<(), IndexError> {
        if content.is_empty() {
            return Err(IndexError::EmptyContent);
        }

        // 处理逻辑
        Ok(())
    }
}

#[derive(Debug)]
pub enum IndexError {
    EmptyContent,
    InvalidId,
    EncodingError,
}
```

**转换要点**：

1. 使用 `Result` 类型处理错误
2. 定义自定义错误类型
3. 使用 `?` 操作符传播错误

---

## 五、并发处理

### 5.1 JavaScript 异步处理

```javascript
// src/async.js
Index.prototype.searchAsync = function(query, limit, options){
    return new Promise(function(resolve, reject){
        try{
            const result = this.search(query, limit, options);
            resolve(result);
        } catch(e){
            reject(e);
        }
    }.bind(this));
};
```

### 5.2 Rust 异步处理

```rust
use tokio::sync::RwLock;

pub struct Index {
    pub map: Arc<RwLock<HashMap<String, Vec<Vec<DocId>>>>,
}

impl Index {
    pub async fn search(&self, query: &str, options: &SearchOptions) -> Result<SearchResults, IndexError> {
        let map = self.map.read().await;
        // 搜索逻辑
        Ok(results)
    }

    pub async fn add(&self, id: DocId, content: &str) -> Result<(), IndexError> {
        let mut map = self.map.write().await;
        // 添加逻辑
        Ok(())
    }
}
```

**转换要点**：

1. 使用 `tokio` 运行时
2. 使用 `Arc<RwLock>` 实现并发访问
3. 使用 `async/await` 语法

---

## 六、测试策略

### 6.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_score() {
        assert_eq!(get_score(9, 10, 0, None, None), 0);
        assert_eq!(get_score(9, 10, 5, None, None), 3);
    }

    #[test]
    fn test_intersect() {
        let arrays = vec![
            vec![vec![1, 2, 3], vec![4, 5, 6]],
            vec![vec![1, 2], vec![3, 4]],
        ];
        let result = intersect(&arrays, 2, 10, 0);
        assert_eq!(result, vec![1, 2]);
    }
}
```

### 6.2 集成测试

```rust
#[tokio::test]
async fn test_search_flow() {
    let index = Index::new(IndexOptions::default());
    index.add(1, "hello world").await.unwrap();
    index.add(2, "hello rust").await.unwrap();

    let results = index.search("hello", &SearchOptions::default()).await.unwrap();
    assert_eq!(results.len(), 2);
}
```

### 6.3 性能测试

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_search(c: &mut Criterion) {
    let index = setup_index();
    c.bench_function("search", |b| {
        b.iter(|| {
            index.search(black_box("hello"), &SearchOptions::default())
        })
    });
}

criterion_group!(benches, bench_search);
criterion_main!(benches);
```

---

## 七、部署考虑

### 7.1 配置管理

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub index: IndexConfig,
    pub cache: CacheConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IndexConfig {
    pub resolution: usize,
    pub tokenize: String,
    pub depth: usize,
}
```

### 7.2 日志记录

```rust
use tracing::{info, error, instrument};

impl Index {
    #[instrument(skip(self))]
    pub async fn search(&self, query: &str, options: &SearchOptions) -> Result<SearchResults, IndexError> {
        info!("Searching for query: {}", query);
        // 搜索逻辑
        Ok(results)
    }
}
```

### 7.3 监控指标

```rust
use prometheus::{Counter, Histogram, IntGauge};

pub struct Metrics {
    pub search_count: Counter,
    pub search_duration: Histogram,
    pub index_size: IntGauge,
}

impl Metrics {
    pub fn record_search(&self, duration: Duration) {
        self.search_count.inc();
        self.search_duration.observe(duration.as_secs_f64());
    }
}
```

---

## 八、总结

从 JavaScript 到 Rust 的转换需要考虑以下关键点：

1. **类型安全**：利用 Rust 的类型系统避免运行时错误
2. **内存管理**：使用所有权和借用机制确保内存安全
3. **性能优化**：利用零成本抽象和编译器优化
4. **错误处理**：使用 `Result` 类型显式处理错误
5. **并发处理**：使用 `tokio` 和 `Arc/RwLock` 实现并发
6. **测试覆盖**：编写完整的单元测试和集成测试
7. **文档完善**：提供清晰的 API 文档和使用示例

通过遵循这些原则，可以构建一个高性能、安全、可维护的 Inversearch 服务。
