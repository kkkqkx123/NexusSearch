# Inversearch 模块映射对照表

## 概述

本文档提供 JavaScript 实现到 Rust 实现的详细模块映射对照表，包括文件、函数、数据结构的对应关系。

---

## 一、核心模块映射

### 1.1 索引模块

| Rust 模块 | JS 文件 | 说明 |
|-----------|---------|------|
| `src/index/mod.rs` | `src/index.js` | 索引核心结构 |
| `src/index/builder.rs` | `src/index/add.js` | 索引构建器 |
| `src/index/remover.rs` | `src/index/remove.js` | 索引删除器 |

#### index/mod.rs 映射

| JS 属性/方法 | Rust 属性/方法 | 类型 | 说明 |
|-------------|---------------|------|------|
| `this.map` | `map` | `HashMap<String, Vec<Vec<DocId>>>` | 倒排索引 |
| `this.ctx` | `ctx` | `HashMap<String, HashMap<String, Vec<Vec<DocId>>>>` | 上下文索引 |
| `this.reg` | `reg` | `KeystoreSet<DocId>` | 文档注册表 |
| `this.resolution` | `resolution` | `usize` | 评分分辨率 |
| `this.tokenize` | `tokenize` | `TokenizeMode` | 分词模式 |
| `this.depth` | `depth` | `usize` | 上下文深度 |
| `this.bidirectional` | `bidirectional` | `bool` | 双向上下文 |
| `this.fastupdate` | `fastupdate` | `bool` | 快速更新 |
| `this.score` | `score` | `Option<Box<dyn ScoreFn>>` | 自定义评分函数 |
| `this.encoder` | `encoder` | `Encoder` | 编码器 |
| `this.compress` | `compress` | `bool` | 压缩选项 |
| `this.rtl` | `rtl` | `bool` | 从右到左 |
| `this.cache` | `cache` | `Option<Cache>` | 缓存 |
| `this.resolve` | `resolve` | `bool` | 解析选项 |
| `this.db` | `db` | `Option<Storage>` | 持久化存储 |
| `this.commit_auto` | `commit_auto` | `bool` | 自动提交 |
| `this.commit_task` | `commit_task` | `Vec<CommitTask>` | 提交任务 |
| `this.commit_timer` | `commit_timer` | `Option<Timer>` | 提交定时器 |
| `this.priority` | `priority` | `usize` | 优先级 |

| JS 方法 | Rust 方法 | 签名 | 说明 |
|---------|-----------|------|------|
| `Index(options)` | `new(options: IndexOptions)` | `-> Self` | 构造函数 |
| `add(id, content, _append, _skip_update)` | `add(&mut self, id: DocId, content: &str)` | `-> Result<(), IndexError>` | 添加文档 |
| `append(id, content)` | `append(&mut self, id: DocId, content: &str)` | `-> Result<(), IndexError>` | 追加文档 |
| `update(id, content)` | `update(&mut self, id: DocId, content: &str)` | `-> Result<(), IndexError>` | 更新文档 |
| `remove(id)` | `remove(&mut self, id: DocId)` | `-> Result<(), IndexError>` | 删除文档 |
| `clear()` | `clear(&mut self)` | `-> ()` | 清空索引 |
| `contain(id)` | `contain(&self, id: DocId)` | `-> bool` | 检查文档 |
| `cleanup()` | `cleanup(&mut self)` | `-> ()` | 清理索引 |
| `mount(db)` | `mount(&mut self, db: Storage)` | `-> Result<(), StorageError>` | 挂载存储 |
| `commit(replace, append)` | `commit(&mut self, replace: bool, append: bool)` | `-> Result<(), StorageError>` | 提交更改 |
| `destroy()` | `destroy(&mut self)` | `-> Result<(), StorageError>` | 销毁存储 |
| `export()` | `export(&self)` | `-> Result<ExportData, ExportError>` | 导出索引 |
| `import(data)` | `import(&mut self, data: ExportData)` | `-> Result<(), ImportError>` | 导入索引 |
| `serialize()` | `serialize(&self)` | `-> Result<String, SerializeError>` | 序列化索引 |

#### index/builder.rs 映射

| JS 函数 | Rust 函数 | 签名 | 说明 |
|---------|-----------|------|------|
| `Index.prototype.add()` | `add()` | `-> Result<(), IndexError>` | 添加文档入口 |
| `push_index()` | `push_index()` | `-> ()` | 推送索引项 |
| `get_score()` | `get_score()` | `-> usize` | 计算评分 |

**评分算法详细映射**：

```javascript
// JavaScript
function get_score(resolution, length, i, term_length, x) {
    return i && (resolution > 1) ? (
        (length + (term_length || 0)) <= resolution ?
            i + (x || 0)
        :
            ((resolution - 1) / (length + (term_length || 0)) * (i + (x || 0)) + 1) | 0
    ) : 0;
}
```

```rust
// Rust
pub fn get_score(
    resolution: usize,
    length: usize,
    i: usize,
    term_length: Option<usize>,
    x: Option<usize>,
) -> usize {
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

**分词模式映射**：

```javascript
// JavaScript
this.tokenize = options.tokenize || "strict";
```

```rust
// Rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenizeMode {
    Strict,
    Forward,
    Reverse,
    Full,
    Bidirectional,
}

impl Default for TokenizeMode {
    fn default() -> Self {
        TokenizeMode::Strict
    }
}
```

### 1.2 搜索模块

| Rust 模块 | JS 文件 | 说明 |
|-----------|---------|------|
| `src/search/mod.rs` | `src/index/search.js` | 搜索核心逻辑 |

#### search/mod.rs 映射

| JS 方法 | Rust 方法 | 签名 | 说明 |
|---------|-----------|------|------|
| `Index.prototype.search()` | `search()` | `-> Result<SearchResults, SearchError>` | 搜索入口 |
| `single_term_query()` | `single_term_query()` | `-> Result<SearchResults, SearchError>` | 单词查询 |
| `get_array()` | `get_array()` | `-> Option<Vec<Vec<DocId>>>` | 获取索引数组 |
| `add_result()` | `add_result()` | `-> Option<Vec<Vec<DocId>>>` | 添加结果 |
| `return_result()` | `return_result()` | `-> SearchResults` | 返回结果 |

**查询流程映射**：

```javascript
// JavaScript
Index.prototype.search = function(query, limit, options) {
    // 1. 参数解析
    if(!options){
        if(!limit && typeof query === "object"){
            options = query;
            query = "";
        }
    }

    // 2. 查询编码
    let query_terms = this.encoder.encode(query, !context);
    let length = query_terms.length;

    // 3. 快速路径：单词语查询
    if(length === 1){
        return single_term_query.call(this, query_terms[0], "", limit, offset, resolve, enrich, tag);
    }

    // 4. 快速路径：单上下文查询
    if(length === 2 && context && !suggest){
        return single_term_query.call(this, query_terms[1], query_terms[0], limit, offset, resolve, enrich, tag);
    }

    // 5. 多词语查询
    for(let arr, term; index < length; index++){
        // ...
    }

    // 6. 返回结果
    return return_result(result, resolution, limit, offset, suggest, boost, resolve);
};
```

```rust
// Rust
impl Index {
    pub fn search(&self, query: &str, options: &SearchOptions) -> Result<SearchResults, SearchError> {
        // 1. 参数解析
        let query = options.query.as_ref().map(|s| s.as_str()).unwrap_or(query);
        let limit = options.limit.unwrap_or(100);
        let offset = options.offset.unwrap_or(0);

        // 2. 查询编码
        let context = self.depth > 0 && options.context.unwrap_or(true);
        let query_terms = self.encoder.encode(query, !context);
        let length = query_terms.len();

        // 3. 快速路径：单词语查询
        if length == 1 {
            return self.single_term_query(&query_terms[0], "", limit, offset);
        }

        // 4. 快速路径：单上下文查询
        if length == 2 && context && !options.suggest.unwrap_or(false) {
            return self.single_term_query(&query_terms[1], &query_terms[0], limit, offset);
        }

        // 5. 多词语查询
        let mut result: Vec<Vec<DocId>> = Vec::new();
        for index in 0..length {
            let term = &query_terms[index];
            let arr = self.get_array(term, keyword)?;
            result = self.add_result(arr, result, options.suggest.unwrap_or(false), resolution)?;
        }

        // 6. 返回结果
        Ok(self.return_result(result, resolution, limit, offset, options.suggest.unwrap_or(false)))
    }
}
```

### 1.3 编码器模块

| Rust 模块 | JS 文件 | 说明 |
|-----------|---------|------|
| `src/encoder/mod.rs` | `src/encoder.js` | 编码器核心逻辑 |
| `src/charset/mod.rs` | `src/charset/` | 字符集规范化 |

#### encoder/mod.rs 映射

| JS 属性/方法 | Rust 属性/方法 | 类型 | 说明 |
|-------------|---------------|------|------|
| `this.normalize` | `normalize` | `NormalizeMode` | 规范化模式 |
| `this.split` | `split` | `Option<Regex>` | 分词正则 |
| `this.prepare` | `prepare` | `Option<Box<dyn Fn(&str) -> String>>` | 预处理函数 |
| `this.finalize` | `finalize` | `Option<Box<dyn Fn(&mut Vec<String>)>>` | 最终处理函数 |
| `this.filter` | `filter` | `Option<HashSet<String>>` | 停用词集合 |
| `this.dedupe` | `dedupe` | `bool` | 去重选项 |
| `this.matcher` | `matcher` | `Option<HashMap<String, String>>` | 词形映射 |
| `this.mapper` | `mapper` | `Option<HashMap<char, char>>` | 字符映射 |
| `this.stemmer` | `stemmer` | `Option<HashMap<String, String>>` | 词干映射 |
| `this.replacer` | `replacer` | `Vec<(Regex, String)>` | 正则替换列表 |
| `this.minlength` | `minlength` | `usize` | 最小长度 |
| `this.maxlength` | `maxlength` | `usize` | 最大长度 |
| `this.rtl` | `rtl` | `bool` | 从右到左 |
| `this.cache` | `cache` | `Option<Cache<String, Vec<String>>>` | 缓存 |
| `this.timer` | `timer` | `Option<Timer>` | 缓存定时器 |
| `this.cache_size` | `cache_size` | `usize` | 缓存大小 |
| `this.cache_enc` | `cache_enc` | `HashMap<String, Vec<String>>` | 编码缓存 |
| `this.cache_term` | `cache_term` | `HashMap<String, String>` | 词缓存 |
| `this.cache_enc_length` | `cache_enc_length` | `usize` | 编码缓存长度 |
| `this.cache_term_length` | `cache_term_length` | `usize` | 词缓存长度 |

| JS 方法 | Rust 方法 | 签名 | 说明 |
|---------|-----------|------|------|
| `Encoder(options)` | `new(options: EncoderOptions)` | `-> Self` | 构造函数 |
| `assign(options)` | `assign(&mut self, options: EncoderOptions)` | `-> &mut Self` | 分配选项 |
| `encode(str, dedupe_terms)` | `encode(&self, str: &str, dedupe_terms: bool)` | `-> Vec<String>` | 编码字符串 |
| `addStemmer(match, replace)` | `add_stemmer(&mut self, match: &str, replace: &str)` | `-> &mut Self` | 添加词干规则 |
| `addFilter(term)` | `add_filter(&mut self, term: &str)` | `-> &mut Self` | 添加停用词 |
| `addMapper(char_match, char_replace)` | `add_mapper(&mut self, char_match: char, char_replace: char)` | `-> &mut Self` | 添加字符映射 |
| `addMatcher(match, replace)` | `add_matcher(&mut self, match: &str, replace: &str)` | `-> &mut Self` | 添加词形映射 |
| `addReplacer(regex, replace)` | `add_replacer(&mut self, regex: Regex, replace: &str)` | `-> &mut Self` | 添加正则替换 |

**编码流程映射**：

```javascript
// JavaScript
Encoder.prototype.encode = function(str, dedupe_terms) {
    // 1. 规范化
    if(this.normalize){
        str = str.normalize("NFKD").replace(normalize, "").toLowerCase();
    }

    // 2. 预处理
    if(this.prepare){
        str = this.prepare(str);
    }

    // 3. 数字分割
    if(this.numeric && str.length > 3){
        str = str.replace(numeric_split_prev_char, "$1 $2")
                 .replace(numeric_split_next_char, "$1 $2")
                 .replace(numeric_split_length, "$1 ");
    }

    // 4. 分词
    let words = this.split ? str.split(this.split) : [str];

    // 5. 处理每个词
    for(let i = 0, word; i < words.length; i++){
        word = words[i];

        // 6. 长度过滤
        if(word.length < this.minlength || word.length > this.maxlength){
            continue;
        }

        // 7. 去重
        if(dedupe_terms && dupes[word]){
            continue;
        }

        // 8. 停用词过滤
        if(this.filter && this.filter.has(word)){
            continue;
        }

        // 9. 词干提取
        if(this.stemmer){
            word = word.replace(this.stemmer_test, match => this.stemmer.get(match));
        }

        // 10. 字符映射
        if(this.mapper){
            // ...
        }

        // 11. 词形归一化
        if(this.matcher){
            word = word.replace(this.matcher_test, match => this.matcher.get(match));
        }

        // 12. 正则替换
        if(this.replacer){
            for(let i = 0; i < this.replacer.length; i+=2){
                word = word.replace(this.replacer[i], this.replacer[i+1]);
            }
        }

        // 13. 添加到结果
        final.push(word);
    }

    // 14. 最终处理
    if(this.finalize){
        final = this.finalize(final) || final;
    }

    return final;
};
```

```rust
// Rust
impl Encoder {
    pub fn encode(&self, str: &str, dedupe_terms: bool) -> Vec<String> {
        let mut result = Vec::new();
        let mut dupes: HashSet<String> = HashSet::new();

        // 1. 规范化
        let mut str = self.normalize_text(str);

        // 2. 预处理
        if let Some(ref prepare) = self.prepare {
            str = prepare(&str);
        }

        // 3. 数字分割
        if self.numeric && str.len() > 3 {
            str = self.split_numeric(&str);
        }

        // 4. 分词
        let words: Vec<&str> = if let Some(ref split) = self.split {
            split.split(&str).collect()
        } else {
            vec![str.as_str()]
        };

        // 5. 处理每个词
        for word in words {
            let word = word.trim();

            // 6. 长度过滤
            if word.len() < self.minlength || word.len() > self.maxlength {
                continue;
            }

            // 7. 去重
            if dedupe_terms && dupes.contains(word) {
                continue;
            }

            let mut word = word.to_string();

            // 8. 停用词过滤
            if let Some(ref filter) = self.filter {
                if filter.contains(&word) {
                    continue;
                }
            }

            // 9. 词干提取
            if let Some(ref stemmer) = self.stemmer {
                word = self.apply_stemmer(&word, stemmer);
            }

            // 10. 字符映射
            if let Some(ref mapper) = self.mapper {
                word = self.apply_mapper(&word, mapper);
            }

            // 11. 词形归一化
            if let Some(ref matcher) = self.matcher {
                word = self.apply_matcher(&word, matcher);
            }

            // 12. 正则替换
            for (regex, replace) in &self.replacer {
                word = regex.replace_all(&word, replace).to_string();
            }

            // 13. 添加到结果
            result.push(word);
        }

        // 14. 最终处理
        if let Some(ref finalize) = self.finalize {
            finalize(&mut result);
        }

        result
    }
}
```

### 1.4 字符集模块

| Rust 模块 | JS 文件 | 说明 |
|-----------|---------|------|
| `src/charset/mod.rs` | `src/charset/normalize.js` | 规范化 |
| `src/charset/latin.rs` | `src/charset/latin/` | 拉丁语编码 |
| `src/charset/cjk.rs` | `src/charset/cjk.js` | CJK 编码 |
| `src/charset/exact.rs` | `src/charset/exact.js` | 精确编码 |

#### charset/mod.rs 映射

| JS 函数 | Rust 函数 | 签名 | 说明 |
|---------|-----------|------|------|
| `normalize_polyfill` | `normalize_polyfill()` | `-> String` | 规范化回退 |
| `normalize` | `normalize()` | `-> String` | Unicode 规范化 |

---

## 二、工具模块映射

### 2.1 通用工具

| Rust 模块 | JS 文件 | 说明 |
|-----------|---------|------|
| `src/common/mod.rs` | `src/common.js` | 通用工具函数 |

#### common/mod.rs 映射

| JS 函数 | Rust 函数 | 签名 | 说明 |
|---------|-----------|------|------|
| `create_object()` | `create_object()` | `-> HashMap<String, Value>` | 创建空对象 |
| `is_array(val)` | `is_array(val: &dyn Any)` | `-> bool` | 判断是否为数组 |
| `is_string(val)` | `is_string(val: &dyn Any)` | `-> bool` | 判断是否为字符串 |
| `is_object(val)` | `is_object(val: &dyn Any)` | `-> bool` | 判断是否为对象 |
| `is_function(val)` | `is_function(val: &dyn Any)` | `-> bool` | 判断是否为函数 |
| `concat(arrays)` | `concat(arrays: Vec<Vec<T>>)` | `-> Vec<T>` | 合并数组 |
| `sort_by_length_down(a, b)` | `sort_by_length_desc(a: &Vec, b: &Vec)` | `-> Ordering` | 按长度降序排序 |
| `sort_by_length_up(a, b)` | `sort_by_length_asc(a: &Vec, b: &Vec)` | `-> Ordering` | 按长度升序排序 |
| `parse_simple(obj, tree)` | `parse_simple(obj: &Value, tree: &Path)` | `-> Option<Value>` | 从对象中提取值 |
| `get_max_len(arr)` | `get_max_len(arr: &[Vec<T>])` | `-> usize` | 获取最大长度 |
| `toArray(val, stringify)` | `to_array(val: &HashSet<T>, stringify: bool)` | `-> Vec<T>` | 转换为数组 |
| `merge_option(value, default_value, merge_value)` | `merge_option()` | `-> T` | 合并选项 |
| `inherit(target_value, default_value)` | `inherit()` | `-> T` | 继承默认值 |

### 2.2 密钥存储

| Rust 模块 | JS 文件 | 说明 |
|-----------|---------|------|
| `src/keystore/mod.rs` | `src/keystore.js` | 密钥存储优化 |

#### keystore/mod.rs 映射

| JS 类 | Rust 结构体 | 说明 |
|--------|------------|------|
| `KeystoreMap` | `KeystoreMap<K, V>` | 密钥存储 Map |
| `KeystoreSet` | `KeystoreSet<T>` | 密钥存储 Set |
| `KeystoreArray` | `KeystoreArray<T>` | 密钥存储数组 |

| JS 方法 | Rust 方法 | 签名 | 说明 |
|---------|-----------|------|------|
| `get(key)` | `get(&self, key: &K)` | `-> Option<&V>` | 获取值 |
| `set(key, value)` | `set(&mut self, key: K, value: V)` | `-> ()` | 设置值 |
| `has(key)` | `contains(&self, key: &K)` | `-> bool` | 检查键 |
| `delete(key)` | `remove(&mut self, key: &K)` | `-> bool` | 删除键 |
| `clear()` | `clear(&mut self)` | `-> ()` | 清空存储 |
| `values()` | `values(&self)` | `-> impl Iterator<Item = &V>` | 获取值迭代器 |
| `keys()` | `keys(&self)` | `-> impl Iterator<Item = &K>` | 获取键迭代器 |
| `entries()` | `iter(&self)` | `-> impl Iterator<Item = (&K, &V)>` | 获取条目迭代器 |

**哈希算法映射**：

```javascript
// JavaScript
function lcg(str) {
    let range = 2 ** this.bit - 1;
    let crc = 0, bit = this.bit + 1;
    for(let i = 0; i < str.length; i++) {
        crc = (crc * bit ^ str.charCodeAt(i)) & range;
    }
    return this.bit === 32 ? crc + 2 ** 31 : crc;
}

function lcg64(str) {
    let range = BigInt(2) ** this.bit - BigInt(1);
    let crc = BigInt(0), bit = this.bit + BigInt(1);
    for(let i = 0; i < str.length; i++){
        crc = (crc * bit ^ BigInt(str.charCodeAt(i))) & range;
    }
    return crc;
}
```

```rust
// Rust
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

fn lcg64(str: &str, bit: u64) -> u64 {
    let range = (1u64 << bit) - 1;
    let mut crc: u64 = 0;
    let bit = bit + 1;

    for byte in str.bytes() {
        crc = (crc.wrapping_mul(bit) ^ byte as u64) & range;
    }

    crc
}
```

---

## 三、结果处理模块映射

### 3.1 交集模块

| Rust 模块 | JS 文件 | 说明 |
|-----------|---------|------|
| `src/intersect/mod.rs` | `src/intersect.js` | 交集和并集计算 |

#### intersect/mod.rs 映射

| JS 函数 | Rust 函数 | 签名 | 说明 |
|---------|-----------|------|------|
| `intersect(arrays, resolution, limit, offset, suggest, boost, resolve)` | `intersect()` | `-> SearchResults` | 计算交集 |
| `union(arrays, limit, offset, resolve, boost)` | `union()` | `-> SearchResults` | 计算并集 |
| `intersect_union(arrays, mandatory, resolve)` | `intersect_union()` | `-> SearchResults` | 计算交集和并集组合 |

### 3.2 高亮模块

| Rust 模块 | JS 文件 | 说明 |
|-----------|---------|------|
| `src/highlight/mod.rs` | `src/document/highlight.js` | 结果高亮 |

#### highlight/mod.rs 映射

| JS 函数 | Rust 函数 | 签名 | 说明 |
|---------|-----------|------|------|
| `highlight_fields()` | `highlight_fields()` | `-> Vec<HighlightedResult>` | 高亮多个字段 |
| `apply_highlight()` | `apply_highlight()` | `-> String` | 应用高亮 |
| `apply_boundary()` | `apply_boundary()` | `-> String` | 应用边界裁剪 |
| `apply_ellipsis()` | `apply_ellipsis()` | `-> String` | 应用省略号 |

### 3.3 解析器模块

| Rust 模块 | JS 文件 | 说明 |
|-----------|---------|------|
| `src/resolver/mod.rs` | `src/resolver.js` | 结果解析器 |

#### resolver/mod.rs 映射

| JS 类/方法 | Rust 结构体/方法 | 签名 | 说明 |
|-----------|-----------------|------|------|
| `Resolver(result, index)` | `new(result: IntermediateSearchResults, index: &Index)` | `-> Self` | 构造函数 |
| `limit(limit)` | `limit(&mut self, limit: usize)` | `-> &mut Self` | 限制结果数量 |
| `offset(offset)` | `offset(&mut self, offset: usize)` | `-> &mut Self` | 设置结果偏移 |
| `boost(boost)` | `boost(&mut self, boost: i32)` | `-> &mut Self` | 提升评分 |
| `resolve()` | `resolve()` | `-> SearchResults` | 解析结果 |
| `execute()` | `execute()` | `-> IntermediateSearchResults` | 执行链式操作 |

---

## 四、存储和缓存模块映射

### 4.1 缓存模块

| Rust 模块 | JS 文件 | 说明 |
|-----------|---------|------|
| `src/cache/mod.rs` | `src/cache.js` | 缓存管理 |

#### cache/mod.rs 映射

| JS 类/方法 | Rust 结构体/方法 | 签名 | 说明 |
|-----------|-----------------|------|------|
| `CacheClass(limit)` | `new(limit: usize)` | `-> Self` | 构造函数 |
| `set(key, value)` | `set(&mut self, key: String, value: T)` | `-> ()` | 设置缓存 |
| `get(key)` | `get(&mut self, key: &str)` | `-> Option<&T>` | 获取缓存 |
| `remove(id)` | `remove(&mut self, id: DocId)` | `-> ()` | 删除缓存 |
| `clear()` | `clear(&mut self)` | `-> ()` | 清空缓存 |

### 4.2 存储模块

| Rust 模块 | JS 文件 | 说明 |
|-----------|---------|------|
| `src/storage/mod.rs` | `src/db/redis/index.js` | Redis 存储 |

#### storage/mod.rs 映射

| JS 方法 | Rust 方法 | 签名 | 说明 |
|---------|-----------|------|------|
| `mount(index)` | `mount(&self, index: &Index)` | `-> Result<(), StorageError>` | 挂载索引 |
| `commit(index, replace, append)` | `commit(&self, index: &Index, replace: bool, append: bool)` | `-> Result<(), StorageError>` | 提交更改 |
| `destroy()` | `destroy(&self)` | `-> Result<(), StorageError>` | 销毁存储 |
| `get(term, keyword, limit, offset, resolve, enrich, tag)` | `get()` | `-> Result<Vec<Vec<DocId>>, StorageError>` | 获取索引数据 |
| `search(index, query_terms, limit, offset, suggest, resolve, enrich, tag)` | `search()` | `-> Result<SearchResults, StorageError>` | 搜索索引 |

---

## 五、类型定义映射

### 5.1 索引选项

| JS 类型 | Rust 结构体 | 说明 |
|---------|------------|------|
| `IndexOptions` | `IndexOptions` | 索引配置选项 |

```javascript
// JavaScript
export let IndexOptions = {
    preset: (string|undefined),
    context: (IndexOptions|undefined),
    encoder: (Encoder|Function|Object|undefined),
    encode: (function(string):Array<string>|undefined),
    resolution: (number|undefined),
    tokenize: (string|undefined),
    fastupdate: (boolean|undefined),
    score: (function():number|undefined),
    keystore: (number|undefined),
    rtl: (boolean|undefined),
    cache: (number|boolean|undefined),
    db: (StorageInterface|undefined),
    commit: (boolean|undefined),
    worker: (string|undefined),
    config: (string|undefined),
    priority: (number|undefined),
};
```

```rust
// Rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexOptions {
    pub preset: Option<String>,
    pub context: Option<ContextOptions>,
    pub encoder: Option<EncoderOptions>,
    pub resolution: Option<usize>,
    pub tokenize: Option<TokenizeMode>,
    pub fastupdate: Option<bool>,
    pub score: Option<Box<dyn ScoreFn>>,
    pub keystore: Option<usize>,
    pub rtl: Option<bool>,
    pub cache: Option<usize>,
    pub commit: Option<bool>,
    pub priority: Option<usize>,
}

impl Default for IndexOptions {
    fn default() -> Self {
        IndexOptions {
            preset: None,
            context: None,
            encoder: None,
            resolution: Some(9),
            tokenize: Some(TokenizeMode::Strict),
            fastupdate: Some(false),
            score: None,
            keystore: None,
            rtl: Some(false),
            cache: None,
            commit: Some(true),
            priority: Some(4),
        }
    }
}
```

### 5.2 搜索选项

| JS 类型 | Rust 结构体 | 说明 |
|---------|------------|------|
| `SearchOptions` | `SearchOptions` | 搜索配置选项 |

```javascript
// JavaScript
export let SearchOptions = {
    query: (string|undefined),
    limit: (number|undefined),
    offset: (number|undefined),
    resolution: (number|undefined),
    context: (boolean|undefined),
    suggest: (boolean|undefined),
    resolve: (boolean|undefined),
    enrich: (boolean|undefined),
    cache: (boolean|undefined)
};
```

```rust
// Rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    pub query: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub resolution: Option<usize>,
    pub context: Option<bool>,
    pub suggest: Option<bool>,
    pub resolve: Option<bool>,
    pub enrich: Option<bool>,
    pub cache: Option<bool>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        SearchOptions {
            query: None,
            limit: Some(100),
            offset: Some(0),
            resolution: None,
            context: None,
            suggest: Some(false),
            resolve: Some(true),
            enrich: Some(false),
            cache: Some(false),
        }
    }
}
```

### 5.3 编码器选项

| JS 类型 | Rust 结构体 | 说明 |
|---------|------------|------|
| `EncoderOptions` | `EncoderOptions` | 编码器配置选项 |

```javascript
// JavaScript
export let EncoderOptions = {
    rtl: (boolean|undefined),
    dedupe: (boolean|undefined),
    include: (EncoderSplitOptions|undefined),
    exclude: (EncoderSplitOptions|undefined),
    split: (string|boolean|RegExp|undefined),
    numeric: (boolean|undefined),
    normalize: (boolean|(function(string):string)|undefined),
    prepare: ((function(string):string)|undefined),
    finalize: ((function(Array<string>):(Array<string>|void))|undefined),
    filter: (Set<string>|function(string):boolean|undefined),
    matcher: (Map<string, string>|undefined),
    mapper: (Map<string, string>|undefined),
    stemmer: (Map<string, string>|undefined),
    replacer: (Array<string|RegExp, string>|undefined),
    minlength: (number|undefined),
    maxlength: (number|undefined),
    cache: (boolean|undefined)
};
```

```rust
// Rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncoderOptions {
    pub rtl: Option<bool>,
    pub dedupe: Option<bool>,
    pub include: Option<EncoderSplitOptions>,
    pub exclude: Option<EncoderSplitOptions>,
    pub split: Option<String>,
    pub numeric: Option<bool>,
    pub normalize: Option<bool>,
    pub prepare: Option<String>,
    pub finalize: Option<String>,
    pub filter: Option<Vec<String>>,
    pub matcher: Option<HashMap<String, String>>,
    pub mapper: Option<HashMap<char, char>>,
    pub stemmer: Option<HashMap<String, String>>,
    pub replacer: Option<Vec<(String, String)>>,
    pub minlength: Option<usize>,
    pub maxlength: Option<usize>,
    pub cache: Option<bool>,
}

impl Default for EncoderOptions {
    fn default() -> Self {
        EncoderOptions {
            rtl: Some(false),
            dedupe: Some(true),
            include: None,
            exclude: None,
            split: None,
            numeric: Some(true),
            normalize: Some(true),
            prepare: None,
            finalize: None,
            filter: None,
            matcher: None,
            mapper: None,
            stemmer: None,
            replacer: None,
            minlength: Some(1),
            maxlength: Some(1024),
            cache: Some(true),
        }
    }
}
```

---

## 六、总结

本映射对照表提供了从 JavaScript 实现到 Rust 实现的完整对应关系，包括：

1. **核心模块**：索引、搜索、编码器、字符集
2. **工具模块**：通用工具、密钥存储
3. **结果处理**：交集、高亮、解析器
4. **存储和缓存**：缓存管理、持久化存储
5. **类型定义**：索引选项、搜索选项、编码器选项

通过参考此映射表，可以确保 Rust 实现与 JavaScript 实现的功能一致性和兼容性。
