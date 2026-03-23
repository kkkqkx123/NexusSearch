# Inversearch 服务分阶段执行方案

## 阶段划分

将 Inversearch 服务的实现分为 5 个阶段，每个阶段都有明确的 JS 实现到 Rust 实现的对应关系。

---

## 阶段一：基础数据结构和工具模块

### 目标

建立基础数据结构和通用工具函数，为后续模块提供支撑。

### 模块映射

| Rust 模块 | JS 参考文件 | 实现内容 |
|-----------|-------------|----------|
| `common` | `src/common.js` | 通用工具函数：类型判断、对象创建、数组操作 |
| `type` | `src/type.js` | 类型定义和结构体 |
| `config` | `src/config.js` | 配置管理和特性开关 |

### 详细对应关系

#### common 模块

**JS 实现**：`src/common.js`

**Rust 实现**：`src/common/mod.rs`

| JS 函数 | Rust 函数 | 说明 |
|---------|-----------|------|
| `create_object()` | `fn create_object() -> HashMap` | 创建空对象 |
| `is_array(val)` | `fn is_array(val: &dyn Any) -> bool` | 判断是否为数组 |
| `is_string(val)` | `fn is_string(val: &dyn Any) -> bool` | 判断是否为字符串 |
| `is_object(val)` | `fn is_object(val: &dyn Any) -> bool` | 判断是否为对象 |
| `concat(arrays)` | `fn concat(arrays: Vec<Vec<T>>) -> Vec<T>` | 合并数组 |
| `sort_by_length_down(a, b)` | `fn sort_by_length_desc(a: &Vec, b: &Vec) -> Ordering` | 按长度降序排序 |
| `sort_by_length_up(a, b)` | `fn sort_by_length_asc(a: &Vec, b: &Vec) -> Ordering` | 按长度升序排序 |
| `parse_simple(obj, tree)` | `fn parse_simple(obj: &Value, tree: &Path) -> Option<Value>` | 从对象中提取值 |

#### type 模块

**JS 实现**：`src/type.js`

**Rust 实现**：`src/type/mod.rs`

| JS 类型定义 | Rust 结构体 | 说明 |
|-------------|------------|------|
| `IndexOptions` | `struct IndexOptions` | 索引配置选项 |
| `SearchOptions` | `struct SearchOptions` | 搜索配置选项 |
| `SearchResults` | `type SearchResults = Vec<DocId>` | 搜索结果 |
| `IntermediateSearchResults` | `type IntermediateSearchResults = Vec<Vec<DocId>>` | 中间搜索结果 |
| `EncoderOptions` | `struct EncoderOptions` | 编码器配置选项 |

#### config 模块

**JS 实现**：`src/config.js`

**Rust 实现**：`src/config/mod.rs`

| JS 常量 | Rust 特性 | 说明 |
|---------|-----------|------|
| `SUPPORT_CACHE` | `feature = "cache"` | 缓存支持 |
| `SUPPORT_ASYNC` | `feature = "async"` | 异步支持 |
| `SUPPORT_STORE` | `feature = "store"` | 存储支持 |
| `SUPPORT_SUGGESTION` | `feature = "suggestion"` | 建议支持 |
| `SUPPORT_KEYSTORE` | `feature = "keystore"` | 密钥存储支持 |

### 验收标准

- 所有通用工具函数实现完成
- 类型定义完整且与 JS 实现一致
- 配置管理支持特性开关

---

## 阶段二：编码器和分词模块

### 目标

实现文本编码和分词功能，支持多种编码策略和语言特性。

### 模块映射

| Rust 模块 | JS 参考文件 | 实现内容 |
|-----------|-------------|----------|
| `encoder` | `src/encoder.js` | 编码器核心逻辑 |
| `charset` | `src/charset/` | 字符集规范化 |
| `tokenizer` | `src/encoder.js` | 分词器（作为 encoder 的子模块） |

### 详细对应关系

#### encoder 模块

**JS 实现**：`src/encoder.js`

**Rust 实现**：`src/encoder/mod.rs`

| JS 方法 | Rust 方法 | 说明 |
|---------|-----------|------|
| `Encoder(options)` | `impl Encoder::new(options: EncoderOptions)` | 编码器构造函数 |
| `encode(str, dedupe_terms)` | `fn encode(&self, str: &str, dedupe: bool) -> Vec<String>` | 编码字符串 |
| `addStemmer(match, replace)` | `fn add_stemmer(&mut self, match: &str, replace: &str)` | 添加词干规则 |
| `addFilter(term)` | `fn add_filter(&mut self, term: &str)` | 添加停用词 |
| `addMapper(char_match, char_replace)` | `fn add_mapper(&mut self, char_match: char, char_replace: char)` | 添加字符映射 |
| `addMatcher(match, replace)` | `fn add_matcher(&mut self, match: &str, replace: &str)` | 添加词形映射 |
| `addReplacer(regex, replace)` | `fn add_replacer(&mut self, regex: Regex, replace: &str)` | 添加正则替换 |

**编码流程对应**：

JS 编码流程（`src/encoder.js` 的 `encode` 方法）：

1. `normalize` - 规范化 → `normalize_text()`
2. `prepare` - 预处理 → `prepare_text()`
3. `numeric` - 数字分割 → `split_numeric()`
4. `split` - 分词 → `split_text()`
5. `filter` - 停用词过滤 → `apply_filter()`
6. `stemmer` - 词干提取 → `apply_stemmer()`
7. `mapper` - 字符映射 → `apply_mapper()`
8. `matcher` - 词形归一化 → `apply_matcher()`
9. `replacer` - 正则替换 → `apply_replacer()`
10. `finalize` - 最终处理 → `finalize()`

#### charset 模块

**JS 实现**：`src/charset/`

**Rust 实现**：`src/charset/mod.rs`

| JS 文件 | Rust 模块 | 说明 |
|---------|-----------|------|
| `src/charset/normalize.js` | `normalize` | Unicode 规范化 |
| `src/charset/latin/advanced.js` | `latin::advanced` | 拉丁语高级编码 |
| `src/charset/latin/balance.js` | `latin::balance` | 拉丁语平衡编码 |
| `src/charset/latin/extra.js` | `latin::extra` | 拉丁语扩展编码 |
| `src/charset/latin/soundex.js` | `latin::soundex` | 拉丁语 Soundex 编码 |
| `src/charset/cjk.js` | `cjk` | CJK 字符处理 |
| `src/charset/exact.js` | `exact` | 精确匹配编码 |

### 验收标准

- 编码器支持所有 JS 实现的编码选项
- 支持多种字符集规范化
- 分词结果与 JS 实现一致
- 性能测试通过

---

## 阶段三：索引管理模块

### 目标

实现倒排索引的创建、更新、删除功能，支持上下文搜索和评分机制。

### 模块映射

| Rust 模块 | JS 参考文件 | 实现内容 |
|-----------|-------------|----------|
| `index` | `src/index.js` | 索引核心结构 |
| `index::builder` | `src/index/add.js` | 索引构建器 |
| `index::remover` | `src/index/remove.js` | 索引删除器 |
| `keystore` | `src/keystore.js` | 密钥存储优化 |

### 详细对应关系

#### index 模块

**JS 实现**：`src/index.js`

**Rust 实现**：`src/index/mod.rs`

| JS 属性/方法 | Rust 属性/方法 | 说明 |
|-------------|---------------|------|
| `this.map` | `map: HashMap<String, Vec<Vec<DocId>>>` | 倒排索引 |
| `this.ctx` | `ctx: HashMap<String, HashMap<String, Vec<Vec<DocId>>>>` | 上下文索引 |
| `this.reg` | `reg: KeystoreSet<DocId>` | 文档注册表 |
| `this.resolution` | `resolution: usize` | 评分分辨率 |
| `this.tokenize` | `tokenize: TokenizeMode` | 分词模式 |
| `this.depth` | `depth: usize` | 上下文深度 |
| `this.bidirectional` | `bidirectional: bool` | 双向上下文 |
| `this.fastupdate` | `fastupdate: bool` | 快速更新 |
| `this.score` | `score: Option<ScoreFn>` | 自定义评分函数 |
| `this.encoder` | `encoder: Encoder` | 编码器 |
| `Index(options)` | `impl Index::new(options: IndexOptions)` | 构造函数 |
| `add(id, content)` | `fn add(&mut self, id: DocId, content: &str)` | 添加文档 |
| `append(id, content)` | `fn append(&mut self, id: DocId, content: &str)` | 追加文档 |
| `update(id, content)` | `fn update(&mut self, id: DocId, content: &str)` | 更新文档 |
| `remove(id)` | `fn remove(&mut self, id: DocId)` | 删除文档 |
| `clear()` | `fn clear(&mut self)` | 清空索引 |
| `contain(id)` | `fn contain(&self, id: DocId) -> bool` | 检查文档是否存在 |

#### index::builder 模块

**JS 实现**：`src/index/add.js`

**Rust 实现**：`src/index/builder.rs`

| JS 函数 | Rust 函数 | 说明 |
|---------|-----------|------|
| `Index.prototype.add()` | `impl Index::add()` | 添加文档入口 |
| `push_index()` | `fn push_index()` | 推送索引项 |
| `get_score()` | `fn get_score()` | 计算评分 |

**评分算法对应**：

JS 实现（`src/index/add.js` 的 `get_score` 函数）：

```javascript
return i && (resolution > 1) ? (
    (length + (term_length || 0)) <= resolution ?
        i + (x || 0)
    :
        ((resolution - 1) / (length + (term_length || 0)) * (i + (x || 0)) + 1) | 0
) : 0;
```

Rust 实现：

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

**分词模式对应**：

JS 实现（`src/index/add.js` 的 switch 语句）：

| JS 模式 | Rust 枚举 | 说明 |
|---------|-----------|------|
| `"strict"` | `TokenizeMode::Strict` | 严格分词 |
| `"forward"` | `TokenizeMode::Forward` | 前向分词 |
| `"reverse"` | `TokenizeMode::Reverse` | 反向分词 |
| `"full"` | `TokenizeMode::Full` | 完整分词 |
| `"bidirectional"` | `TokenizeMode::Bidirectional` | 双向分词 |

#### keystore 模块

**JS 实现**：`src/keystore.js`

**Rust 实现**：`src/keystore/mod.rs`

| JS 类 | Rust 结构体 | 说明 |
|--------|------------|------|
| `KeystoreMap` | `struct KeystoreMap<K, V>` | 密钥存储 Map |
| `KeystoreSet` | `struct KeystoreSet<T>` | 密钥存储 Set |
| `KeystoreArray` | `struct KeystoreArray<T>` | 密钥存储数组 |

| JS 方法 | Rust 方法 | 说明 |
|---------|-----------|------|
| `get(key)` | `fn get(&self, key: &K) -> Option<&V>` | 获取值 |
| `set(key, value)` | `fn set(&mut self, key: K, value: V)` | 设置值 |
| `has(key)` | `fn contains(&self, key: &K) -> bool` | 检查键是否存在 |
| `delete(key)` | `fn remove(&mut self, key: &K) -> bool` | 删除键 |
| `clear()` | `fn clear(&mut self)` | 清空存储 |
| `values()` | `fn values(&self) -> impl Iterator<Item = &V>` | 获取值迭代器 |
| `keys()` | `fn keys(&self) -> impl Iterator<Item = &K>` | 获取键迭代器 |

**哈希算法对应**：

JS 实现（`src/keystore.js` 的 `lcg` 和 `lcg64` 函数）：

| JS 函数 | Rust 函数 | 说明 |
|---------|-----------|------|
| `lcg(str)` | `fn lcg(str: &str, bit: u32) -> u32` | 32 位线性同余生成器 |
| `lcg64(str)` | `fn lcg64(str: &str, bit: u64) -> u64` | 64 位线性同余生成器 |

### 验收标准

- 索引创建、更新、删除功能完整
- 支持所有分词模式
- 评分算法与 JS 实现一致
- 上下文搜索功能正常
- 快速更新模式工作正常

---

## 阶段四：搜索和结果处理模块

### 目标

实现查询解析、结果获取、排序、高亮等功能。

### 模块映射

| Rust 模块 | JS 参考文件 | 实现内容 |
|-----------|-------------|----------|
| `search` | `src/index/search.js` | 搜索核心逻辑 |
| `intersect` | `src/intersect.js` | 交集和并集计算 |
| `highlight` | `src/document/highlight.js` | 结果高亮 |
| `resolver` | `src/resolver.js` | 结果解析器 |

### 详细对应关系

#### search 模块

**JS 实现**：`src/index/search.js`

**Rust 实现**：`src/search/mod.rs`

| JS 方法 | Rust 方法 | 说明 |
|---------|-----------|------|
| `Index.prototype.search()` | `impl Index::search()` | 搜索入口 |
| `single_term_query()` | `fn single_term_query()` | 单词查询 |
| `get_array()` | `fn get_array()` | 获取索引数组 |
| `add_result()` | `fn add_result()` | 添加结果 |
| `return_result()` | `fn return_result()` | 返回结果 |

**查询流程对应**：

JS 查询流程（`src/index/search.js` 的 `search` 方法）：

1. 参数解析 → `parse_search_options()`
2. 查询编码 → `encoder.encode()`
3. 单词查询快速路径 → `single_term_query()`
4. 多词查询 → `multi_term_query()`
5. 结果排序 → `sort_results()`
6. 结果分页 → `paginate_results()`

#### intersect 模块

**JS 实现**：`src/intersect.js`

**Rust 实现**：`src/intersect/mod.rs`

| JS 函数 | Rust 函数 | 说明 |
|---------|-----------|------|
| `intersect(arrays, resolution, limit, offset, suggest, boost, resolve)` | `fn intersect()` | 计算交集 |
| `union(arrays, limit, offset, resolve, boost)` | `fn union()` | 计算并集 |
| `intersect_union(arrays, mandatory, resolve)` | `fn intersect_union()` | 计算交集和并集组合 |

**交集算法对应**：

JS 实现（`src/intersect.js` 的 `intersect` 函数）：

```javascript
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
```

Rust 实现：

```rust
fn intersect(arrays: &[IntermediateSearchResults], resolution: usize, limit: usize, offset: usize) -> SearchResults {
    let mut check: HashMap<DocId, usize> = HashMap::new();
    let mut result: Vec<Vec<DocId>> = Vec::new();
    let length = arrays.len();

    for y in 0..resolution {
        for x in 0..length {
            if let Some(ids) = arrays.get(x).and_then(|arr| arr.get(y)) {
                for &id in ids {
                    let count = check.entry(id).or_insert(0);
                    *count += 1;
                    let slot = result.get_mut(*count).unwrap();
                    slot.push(id);
                }
            }
        }
    }

    // 返回出现次数等于查询词数的结果
    result.get(length).cloned().unwrap_or_default()
}
```

#### highlight 模块

**JS 实现**：`src/document/highlight.js`

**Rust 实现**：`src/highlight/mod.rs`

| JS 函数 | Rust 函数 | 说明 |
|---------|-----------|------|
| `highlight_fields()` | `fn highlight_fields()` | 高亮多个字段 |
| `apply_highlight()` | `fn apply_highlight()` | 应用高亮 |
| `apply_boundary()` | `fn apply_boundary()` | 应用边界裁剪 |
| `apply_ellipsis()` | `fn apply_ellipsis()` | 应用省略号 |

**高亮流程对应**：

JS 高亮流程（`src/document/highlight.js` 的 `highlight_fields` 函数）：

1. 解析模板 → `parse_template()`
2. 编码查询 → `encoder.encode()`
3. 检测匹配 → `detect_matches()`
4. 应用高亮 → `apply_highlight()`
5. 应用边界 → `apply_boundary()`
6. 应用省略号 → `apply_ellipsis()`

#### resolver 模块

**JS 实现**：`src/resolver.js`

**Rust 实现**：`src/resolver/mod.rs`

| JS 类/方法 | Rust 结构体/方法 | 说明 |
|-----------|-----------------|------|
| `Resolver(result, index)` | `struct Resolver` | 解析器构造函数 |
| `limit(limit)` | `fn limit(&mut self, limit: usize)` | 限制结果数量 |
| `offset(offset)` | `fn offset(&mut self, offset: usize)` | 设置结果偏移 |
| `boost(boost)` | `fn boost(&mut self, boost: i32)` | 提升评分 |
| `resolve()` | `fn resolve()` | 解析结果 |
| `execute()` | `fn execute()` | 执行链式操作 |

**链式操作对应**：

JS 实现（`src/resolve/` 目录）：

| JS 文件 | Rust 模块 | 说明 |
|---------|-----------|------|
| `src/resolve/default.js` | `resolver::default` | 默认解析 |
| `src/resolve/and.js` | `resolver::and` | AND 操作 |
| `src/resolve/or.js` | `resolver::or` | OR 操作 |
| `src/resolve/xor.js` | `resolver::xor` | XOR 操作 |
| `src/resolve/not.js` | `resolver::not` | NOT 操作 |

### 验收标准

- 搜索功能与 JS 实现一致
- 交集和并集计算正确
- 高亮功能正常工作
- 链式操作支持完整
- 查询建议功能正常

---

## 阶段五：缓存、存储和集成

### 目标

实现缓存管理、持久化存储，完成服务集成和 gRPC 接口。

### 模块映射

| Rust 模块 | JS 参考文件 | 实现内容 |
|-----------|-------------|----------|
| `cache` | `src/cache.js` | 缓存管理 |
| `storage` | `src/db/redis/index.js` | Redis 存储 |
| `grpc` | - | gRPC 服务接口 |
| `main` | `src/index.js` | 服务入口 |

### 详细对应关系

#### cache 模块

**JS 实现**：`src/cache.js`

**Rust 实现**：`src/cache/mod.rs`

| JS 类/方法 | Rust 结构体/方法 | 说明 |
|-----------|-----------------|------|
| `CacheClass(limit)` | `struct Cache<T>` | 缓存构造函数 |
| `set(key, value)` | `fn set(&mut self, key: String, value: T)` | 设置缓存 |
| `get(key)` | `fn get(&mut self, key: &str) -> Option<&T>` | 获取缓存 |
| `remove(id)` | `fn remove(&mut self, id: DocId)` | 删除缓存 |
| `clear()` | `fn clear(&mut self)` | 清空缓存 |

**LRU 淘汰策略对应**：

JS 实现（`src/cache.js` 的 `set` 方法）：

```javascript
CacheClass.prototype.set = function(key, value){
    this.cache.set(this.last = key, value);
    if(this.cache.size > this.limit){
        this.cache.delete(this.cache.keys().next().value);
    }
};
```

Rust 实现：

```rust
impl<T> Cache<T> {
    pub fn set(&mut self, key: String, value: T) {
        self.cache.insert(key.clone(), value);
        self.last = key;

        if self.cache.len() > self.limit {
            if let Some(oldest) = self.cache.keys().next() {
                self.cache.remove(oldest);
            }
        }
    }
}
```

#### storage 模块

**JS 实现**：`src/db/redis/index.js`

**Rust 实现**：`src/storage/mod.rs`

| JS 方法 | Rust 方法 | 说明 |
|---------|-----------|------|
| `mount(index)` | `fn mount(&self, index: &Index)` | 挂载索引 |
| `commit(index, replace, append)` | `fn commit(&self, index: &Index, replace: bool, append: bool)` | 提交更改 |
| `destroy()` | `fn destroy(&self)` | 销毁存储 |
| `get(term, keyword, limit, offset, resolve, enrich, tag)` | `fn get()` | 获取索引数据 |
| `search(index, query_terms, limit, offset, suggest, resolve, enrich, tag)` | `fn search()` | 搜索索引 |

#### grpc 模块

**Rust 实现**：`src/grpc/mod.rs`

| gRPC 方法 | 说明 |
|-----------|------|
| `AddDocument` | 添加文档 |
| `UpdateDocument` | 更新文档 |
| `RemoveDocument` | 删除文档 |
| `Search` | 搜索文档 |
| `ClearIndex` | 清空索引 |
| `GetStats` | 获取统计信息 |

#### main 模块

**JS 实现**：`src/index.js`

**Rust 实现**：`src/main.rs`

| JS 功能 | Rust 功能 | 说明 |
|---------|-----------|------|
| `Index` 构造函数 | `main()` 函数 | 服务入口 |
| 配置加载 | `load_config()` | 加载配置 |
| 服务启动 | `start_server()` | 启动 gRPC 服务 |

### 验收标准

- 缓存功能正常工作
- Redis 存储集成完成
- gRPC 接口完整
- 服务可以正常启动和停止
- 端到端测试通过

---

## 总体验收标准

1. 所有模块实现完成，与 JS 实现功能一致
2. 单元测试覆盖率 > 80%
3. 集成测试通过
4. 性能测试：搜索延迟 < 10ms（单次查询）
5. 内存使用：与 JS 实现相比减少 > 30%
6. 文档完整：API 文档、架构文档、部署文档

---

## 依赖关系

```
阶段一（基础）
  ↓
阶段二（编码器）
  ↓
阶段三（索引）
  ↓
阶段四（搜索）
  ↓
阶段五（集成）
```

各阶段必须按顺序完成，后续阶段依赖前一阶段的实现。
