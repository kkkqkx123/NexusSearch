# Inversearch 服务设计

## 模块概述

Inversearch 服务是基于 FlexSearch 核心搜索逻辑的 Rust 实现，提供高性能的关键词搜索、模糊匹配、短语搜索和结果高亮功能。

## 模块列表

| 模块 | 职责 | JS 参考文件 |
|------|------|-------------|
| **index** | 核心索引管理器，处理倒排索引的创建、更新和删除 | src/index.js, src/index/add.js, src/index/remove.js |
| **search** | 搜索管理器，处理查询解析、结果排序和返回 | src/index/search.js |
| **tokenizer** | 分词器，处理文本分词、规范化、编码 | src/encoder.js, src/charset/ |
| **highlight** | 结果高亮显示，支持多种高亮模式 | src/document/highlight.js |
| **cache** | 缓存管理，提高查询性能 | src/cache.js |
| **storage** | 持久化存储，支持 Redis | src/db/redis/index.js |
| **intersect** | 结果交集和并集计算 | src/intersect.js |
| **resolver** | 结果解析器，支持链式操作 | src/resolver.js |

---

## 模块职责

### index 模块

**核心职责**：
- 管理倒排索引的创建、更新和删除
- 支持上下文搜索（Context Search）
- 支持多种分词模式（strict、forward、reverse、full、bidirectional）
- 实现评分机制（基于位置和分辨率）
- 支持快速更新模式（Fast Update）

**参考实现**：
- `src/index.js` - 索引核心结构、配置管理、索引操作接口
- `src/index/add.js` - 文档添加逻辑，包括分词、评分、索引构建
- `src/index/remove.js` - 文档删除逻辑，支持快速更新

**主要功能**：
1. 文档索引：将文档内容分词并建立倒排索引
2. 上下文索引：支持基于词序的上下文搜索
3. 评分计算：根据词位置和文档长度计算相关性得分
4. 索引更新：支持添加、更新、删除操作
5. 快速更新：维护文档引用，加速更新操作

---

### search 模块

**核心职责**：
- 处理查询解析和编码
- 支持单词查询、多词查询、上下文查询
- 实现查询建议（Suggestion）
- 支持结果分页和偏移
- 处理查询缓存

**参考实现**：
- `src/index/search.js` - 搜索核心逻辑，包括查询处理、结果获取、排序

**主要功能**：
1. 查询解析：解析查询字符串，处理特殊字符
2. 单词查询：快速路径处理单个词的查询
3. 多词查询：处理多个词的交集查询
4. 上下文查询：支持基于词序的上下文搜索
5. 查询建议：当无结果时提供近似查询建议
6. 结果排序：根据评分和位置对结果排序

---

### tokenizer 模块

**核心职责**：
- 文本预处理和规范化
- 支持多种分词策略
- 实现停用词过滤
- 支持词干提取（Stemmer）
- 支持字符映射（Mapper）
- 支持词形归一化（Matcher）

**参考实现**：
- `src/encoder.js` - 编码器核心逻辑，包括预处理、分词、过滤
- `src/charset/` - 字符集规范化，包括拉丁语、CJK、精确匹配等

**主要功能**：
1. 文本规范化：大小写转换、Unicode 规范化
2. 分词：基于正则表达式或自定义规则分词
3. 停用词过滤：过滤常见无意义词
4. 词干提取：将词还原为词干形式
5. 字符映射：字符替换和转换
6. 词形归一化：将不同形式的词映射到统一形式

---

### highlight 模块

**核心职责**：
- 在搜索结果中高亮显示匹配的关键词
- 支持自定义高亮模板
- 支持边界裁剪（Boundary Clipping）
- 支持省略号（Ellipsis）
- 支持多字段高亮

**参考实现**：
- `src/document/highlight.js` - 高亮核心逻辑，包括匹配检测、边界处理、模板应用

**主要功能**：
1. 匹配检测：在文档内容中检测查询词匹配
2. 高亮标记：使用模板标记匹配的文本
3. 边界处理：控制高亮区域的长度和位置
4. 省略号：在截断位置添加省略号
5. 多字段：支持在多个字段中分别高亮

---

### cache 模块

**核心职责**：
- 实现查询结果缓存
- 支持 LRU 淘汰策略
- 支持缓存大小限制
- 支持缓存失效和清理

**参考实现**：
- `src/cache.js` - 缓存核心逻辑，包括缓存存储、淘汰策略

**主要功能**：
1. 缓存存储：存储查询结果
2. 缓存命中：快速返回缓存结果
3. 淘汰策略：LRU 淘汰旧缓存
4. 缓存失效：文档更新时失效相关缓存
5. 缓存清理：定期清理过期缓存

---

### storage 模块

**核心职责**：
- 实现索引持久化存储
- 支持 Redis 作为后端存储
- 支持索引导出和导入
- 支持增量更新

**参考实现**：
- `src/db/redis/index.js` - Redis 存储接口，包括索引存储、查询、更新
- `src/serialize.js` - 索引序列化和反序列化

**主要功能**：
1. 索引存储：将索引数据持久化到 Redis
2. 索引查询：从 Redis 读取索引数据
3. 索引更新：增量更新 Redis 中的索引
4. 索引导出：导出索引数据
5. 索引导入：导入索引数据

---

### intersect 模块

**核心职责**：
- 计算多个查询结果的交集
- 计算多个查询结果的并集
- 支持加权交集和并集
- 支持结果去重

**参考实现**：
- `src/intersect.js` - 交集和并集核心逻辑

**主要功能**：
1. 交集计算：计算多个结果集的交集
2. 并集计算：计算多个结果集的并集
3. 加权计算：支持不同结果集的权重
4. 结果去重：去除重复的结果
5. 评分聚合：聚合多个结果的评分

---

### resolver 模块

**核心职责**：
- 支持链式查询操作
- 支持 AND、OR、XOR、NOT 操作
- 支持结果增强（Enrich）
- 支持结果限制和偏移

**参考实现**：
- `src/resolver.js` - 解析器核心逻辑
- `src/resolve/` - 各种解析操作实现

**主要功能**：
1. 链式操作：支持链式调用查询方法
2. 布尔操作：支持 AND、OR、XOR、NOT
3. 结果增强：从存储中获取完整文档
4. 结果限制：限制返回结果数量
5. 结果偏移：跳过指定数量的结果

---

## 数据结构设计

### 倒排索引

基于 `src/index.js` 和 `src/index/add.js` 的实现：

```
Map<Term, Array<ResolutionSlot>>
ResolutionSlot: Array<DocID>
```

- Term：分词后的词
- ResolutionSlot：按评分分槽的文档 ID 数组
- DocID：文档唯一标识符

### 上下文索引

基于 `src/index.js` 的上下文搜索实现：

```
Map<Keyword, Map<Term, Array<ResolutionSlot>>>
```

- Keyword：上下文关键词
- Term：当前词
- ResolutionSlot：按评分分槽的文档 ID 数组

### 文档注册表

基于 `src/index.js` 和 `src/keystore.js` 的实现：

```
Map<DocID, Array<ResolutionSlot>>
```

- DocID：文档唯一标识符
- ResolutionSlot：文档引用的索引槽，用于快速更新

---

## 关键算法

### 评分算法

基于 `src/index/add.js` 的 `get_score` 函数：

评分基于词在文档中的位置和文档长度，公式为：

```
score = (resolution - 1) / (document_length + term_length) * (position + offset)
```

- resolution：分辨率，控制评分精度
- document_length：文档总词数
- term_length：当前词长度
- position：词在文档中的位置
- offset：偏移量（用于部分匹配）

### 查询算法

基于 `src/index/search.js` 的实现：

1. **单词查询**：直接从倒排索引中获取结果
2. **多词查询**：对每个词获取结果，然后计算交集
3. **上下文查询**：使用上下文索引，关键词和当前词的组合查询
4. **查询建议**：当无结果时，逐步放宽查询条件

### 交集算法

基于 `src/intersect.js` 的实现：

使用哈希表统计每个文档 ID 出现的次数，只返回出现次数等于查询词数的文档。

---

## 性能优化

### 内存优化

基于 `src/keystore.js` 的实现：

- 使用分片 Map 存储大量键值对
- 使用线性同余生成器（LCG）进行哈希
- 支持大容量存储（超过 2^31 个元素）

### 查询优化

基于 `src/index/search.js` 的实现：

- 快速路径处理单词语查询
- 提前终止：当达到结果数量限制时提前返回
- 查询缓存：缓存常用查询结果

### 索引优化

基于 `src/index/add.js` 的实现：

- 快速更新：维护文档引用，加速更新操作
- 去重：避免重复索引相同的词
- 评分分槽：按评分分槽存储，加速排序

---

## 配置选项

基于 `src/config.js` 和 `src/type.js` 的实现：

| 选项 | 类型 | 说明 |
|------|------|------|
| resolution | number | 评分分辨率，默认 9 |
| tokenize | string | 分词模式，默认 "strict" |
| depth | number | 上下文深度，默认 0 |
| bidirectional | boolean | 是否支持双向上下文，默认 true |
| fastupdate | boolean | 是否启用快速更新，默认 false |
| cache | number/boolean | 缓存配置，默认 false |
| rtl | boolean | 是否从右到左处理，默认 false |

---

## 与其他服务的交互

### 与查询协调器的交互

- 接收 gRPC 查询请求
- 返回搜索结果
- 支持批量查询

### 与存储层的交互

- 将索引数据持久化到 Redis
- 从 Redis 加载索引数据
- 支持增量更新

### 与缓存层的交互

- 使用 Redis 作为查询缓存
- 缓存热门查询结果
- 支持缓存失效
