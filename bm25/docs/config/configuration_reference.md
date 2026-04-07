# BM25 服务配置项参考

## 完整配置示例

```toml
# BM25 服务配置文件
# 文档版本：1.0
# 最后更新：2026-04-07

# ============================================================================
# 服务器配置
# ============================================================================
[server]
# 服务监听地址
# 格式：IP:PORT
# 默认：0.0.0.0:50051
# 环境变量：SERVER_ADDRESS
address = "0.0.0.0:50051"

# ============================================================================
# 存储配置
# ============================================================================
[storage]
# 存储类型：tantivy 或 redis
# 默认：tantivy
# 环境变量：STORAGE_TYPE
type = "tantivy"

# Tantivy 存储配置
[storage.tantivy]
# 索引路径
# 默认：./index
# 环境变量：TANTIVY_INDEX_PATH
index_path = "./index"

# 写入器内存预算（MB）
# 范围：10-500
# 默认：50
# 环境变量：TANTIVY_WRITER_MEMORY_MB
writer_memory_mb = 50

# Redis 存储配置
[storage.redis]
# Redis 连接 URL
# 默认：redis://127.0.0.1:6379
# 环境变量：REDIS_URL
url = "redis://127.0.0.1:6379"

# 连接池大小
# 范围：1-100
# 默认：10
# 环境变量：REDIS_POOL_SIZE
pool_size = 10

# 连接超时时间（秒）
# 默认：5
# 环境变量：REDIS_CONNECTION_TIMEOUT_SECS
connection_timeout_secs = 5

# Key 前缀
# 默认：bm25
# 环境变量：REDIS_KEY_PREFIX
key_prefix = "bm25"

# 最小空闲连接数
# 默认：2
# 环境变量：REDIS_MIN_IDLE
min_idle = 2

# 连接最大生命周期（秒）
# 默认：60
# 环境变量：REDIS_MAX_LIFETIME_SECS
max_lifetime_secs = 60

# ============================================================================
# 索引配置
# ============================================================================
[index]
# 数据目录
# 默认：./data
# 环境变量：DATA_DIR
data_dir = "./data"

# 索引路径
# 默认：./index
# 环境变量：INDEX_PATH
index_path = "./index"

# ============================================================================
# Tantivy 索引管理器配置
# ============================================================================
[index.manager]
# 写入器内存预算（字节）
# 范围：1_000_000 - 500_000_000
# 默认：50_000_000 (50MB)
# 环境变量：INDEX_WRITER_MEMORY_BUDGET
writer_memory_budget = 50000000

# 写入器线程数
# 0 表示自动检测 CPU 核心数
# 范围：0-32
# 默认：0
# 环境变量：INDEX_WRITER_THREADS
writer_num_threads = 0

# 是否启用 Reader 缓存
# 默认：true
# 环境变量：INDEX_READER_CACHE
reader_cache_enabled = true

# Reader 重载策略
# 可选值：manual, on_commit_with_delay
# 默认：on_commit_with_delay（推荐）
# 环境变量：INDEX_READER_RELOAD_POLICY
reader_reload_policy = "on_commit_with_delay"

# 合并策略类型
# 可选值：log, no_merge
# 默认：log（推荐）
# 环境变量：INDEX_MERGE_POLICY
merge_policy = "log"

# LogMergePolicy 详细配置
[index.manager.log_merge_policy]
# 最小合并段数
# 范围：2-20
# 默认：8
# 环境变量：INDEX_LOG_MERGE_MIN_NUM_SEGMENTS
min_num_segments = 8

# 合并前最大文档数
# 范围：100_000 - 100_000_000
# 默认：10_000_000
# 环境变量：INDEX_LOG_MERGE_MAX_DOCS_BEFORE_MERGE
max_docs_before_merge = 10000000

# 最小层大小
# 范围：1_000 - 1_000_000
# 默认：10_000
# 环境变量：INDEX_LOG_MERGE_MIN_LAYER_SIZE
min_layer_size = 10000

# 层大小对数比率
# 范围：0.1-1.0
# 默认：0.75
# 环境变量：INDEX_LOG_MERGE_LEVEL_LOG_SIZE
level_log_size = 0.75

# 合并前删除文档比率
# 范围：0.0-1.0
# 默认：1.0
# 环境变量：INDEX_LOG_MERGE_DEL_DOCS_RATIO_BEFORE_MERGE
del_docs_ratio_before_merge = 1.0

# ============================================================================
# BM25 算法配置
# ============================================================================
[bm25]
# BM25 k1 参数（控制词频饱和度）
# 范围：0.0-10.0
# 推荐值：1.2-2.0
# 默认：1.2
# 环境变量：BM25_K1
k1 = 1.2

# BM25 b 参数（控制文档长度归一化）
# 范围：0.0-1.0
# 推荐值：0.5-0.8
# 默认：0.75
# 环境变量：BM25_B
b = 0.75

# 平均文档长度
# 范围：1.0-10000.0
# 默认：100.0
# 环境变量：BM25_AVG_DOC_LENGTH
avg_doc_length = 100.0

# 字段权重配置
[bm25.field_weights]
# 标题字段权重
# 范围：0.0-10.0
# 默认：2.0
# 环境变量：BM25_TITLE_WEIGHT
title = 2.0

# 内容字段权重
# 范围：0.0-10.0
# 默认：1.0
# 环境变量：BM25_CONTENT_WEIGHT
content = 1.0

# ============================================================================
# 搜索配置
# ============================================================================
[search]
# 默认返回结果数量
# 范围：1-1000
# 默认：10
# 环境变量：SEARCH_DEFAULT_LIMIT
default_limit = 10

# 最大返回结果数量
# 范围：1-10000
# 默认：100
# 环境变量：SEARCH_MAX_LIMIT
max_limit = 100

# 是否启用高亮
# 默认：true
# 环境变量：SEARCH_ENABLE_HIGHLIGHT
enable_highlight = true

# 高亮片段大小（字符数）
# 范围：50-1000
# 默认：200
# 环境变量：SEARCH_HIGHLIGHT_FRAGMENT_SIZE
highlight_fragment_size = 200

# 是否启用拼写检查
# 默认：false
# 环境变量：SEARCH_ENABLE_SPELL_CHECK
enable_spell_check = false

# 是否启用模糊匹配
# 默认：false
# 环境变量：SEARCH_FUZZY_MATCHING
fuzzy_matching = false

# 模糊匹配距离（编辑距离）
# 范围：1-10
# 默认：2
# 环境变量：SEARCH_FUZZY_DISTANCE
fuzzy_distance = 2
```

---

## 配置项详细说明

### 服务器配置

#### `server.address`

服务监听的网络地址和端口。

- **类型**: String
- **格式**: `IP:PORT`
- **默认值**: `0.0.0.0:50051`
- **环境变量**: `SERVER_ADDRESS`
- **示例**: 
  - `0.0.0.0:50051` - 监听所有网卡
  - `127.0.0.1:50051` - 仅监听本地
  - `:::50051` - IPv6 监听

---

### 存储配置

#### `storage.type`

指定使用的存储后端类型。

- **类型**: String
- **可选值**: `tantivy`, `redis`
- **默认值**: `tantivy`
- **环境变量**: `STORAGE_TYPE`
- **说明**:
  - `tantivy`: 使用 Tantivy 搜索引擎作为存储后端（推荐用于全文搜索）
  - `redis`: 使用 Redis 作为存储后端（推荐用于缓存或分布式场景）

#### `storage.tantivy.index_path`

Tantivy 索引文件的存储路径。

- **类型**: String
- **默认值**: `./index`
- **环境变量**: `TANTIVY_INDEX_PATH`

#### `storage.tantivy.writer_memory_mb`

Tantivy 写入器的内存预算（单位：MB）。

- **类型**: Integer
- **范围**: 10-500
- **默认值**: 50
- **环境变量**: `TANTIVY_WRITER_MEMORY_MB`
- **说明**:
  - 值越大，索引速度越快，但内存占用越高
  - 建议根据文档数量和可用内存调整

#### `storage.redis.url`

Redis 服务器连接 URL。

- **类型**: String
- **格式**: `redis://[host]:[port]`
- **默认值**: `redis://127.0.0.1:6379`
- **环境变量**: `REDIS_URL`

#### `storage.redis.pool_size`

Redis 连接池大小。

- **类型**: Integer
- **范围**: 1-100
- **默认值**: 10
- **环境变量**: `REDIS_POOL_SIZE`

---

### 索引配置

#### `index.data_dir`

数据存储目录。

- **类型**: String
- **默认值**: `./data`
- **环境变量**: `DATA_DIR`

#### `index.index_path`

索引存储路径。

- **类型**: String
- **默认值**: `./index`
- **环境变量**: `INDEX_PATH`

---

### Tantivy 索引管理器配置

#### `index.manager.writer_memory_budget`

写入器内存预算（单位：字节）。

- **类型**: Integer
- **范围**: 1_000_000 - 500_000_000
- **默认值**: 50_000_000 (50MB)
- **环境变量**: `INDEX_WRITER_MEMORY_BUDGET`

#### `index.manager.writer_num_threads`

写入器线程数。

- **类型**: Integer
- **范围**: 0-32
- **默认值**: 0 (自动检测 CPU 核心数)
- **环境变量**: `INDEX_WRITER_THREADS`

#### `index.manager.reader_cache_enabled`

是否启用 Reader 缓存。

- **类型**: Boolean
- **默认值**: true
- **环境变量**: `INDEX_READER_CACHE`

#### `index.manager.reader_reload_policy`

Reader 重载策略。

- **类型**: String
- **可选值**: 
  - `manual`: 手动重载
  - `on_commit_with_delay`: 提交后延迟重载（推荐）
- **默认值**: `on_commit_with_delay`
- **环境变量**: `INDEX_READER_RELOAD_POLICY`

#### `index.manager.merge_policy`

合并策略类型。

- **类型**: String
- **可选值**: 
  - `log`: 对数合并策略（推荐）
  - `no_merge`: 不合并（仅用于测试）
- **默认值**: `log`
- **环境变量**: `INDEX_MERGE_POLICY`

---

### BM25 算法配置

#### `bm25.k1`

BM25 的 k1 参数，控制词频饱和度。

- **类型**: Float
- **范围**: 0.0-10.0
- **推荐值**: 1.2-2.0
- **默认值**: 1.2
- **环境变量**: `BM25_K1`
- **说明**:
  - 值越大，词频对得分的影响越大
  - 值越小，词频饱和度越高（即词频达到一定值后影响不再增加）

#### `bm25.b`

BM25 的 b 参数，控制文档长度归一化。

- **类型**: Float
- **范围**: 0.0-1.0
- **推荐值**: 0.5-0.8
- **默认值**: 0.75
- **环境变量**: `BM25_B`
- **说明**:
  - 值越大，文档长度归一化影响越大
  - 值为 0 时，不考虑文档长度影响
  - 值为 1 时，完全考虑文档长度影响

#### `bm25.avg_doc_length`

平均文档长度。

- **类型**: Float
- **范围**: 1.0-10000.0
- **默认值**: 100.0
- **环境变量**: `BM25_AVG_DOC_LENGTH`
- **说明**: 用于 BM25 算法中的文档长度归一化

#### `bm25.field_weights.title`

标题字段的权重。

- **类型**: Float
- **范围**: 0.0-10.0
- **默认值**: 2.0
- **环境变量**: `BM25_TITLE_WEIGHT`
- **说明**: 值越大，标题匹配对得分的贡献越大

#### `bm25.field_weights.content`

内容字段的权重。

- **类型**: Float
- **范围**: 0.0-10.0
- **默认值**: 1.0
- **环境变量**: `BM25_CONTENT_WEIGHT`

---

### 搜索配置

#### `search.default_limit`

默认返回结果数量。

- **类型**: Integer
- **范围**: 1-1000
- **默认值**: 10
- **环境变量**: `SEARCH_DEFAULT_LIMIT`

#### `search.max_limit`

最大返回结果数量。

- **类型**: Integer
- **范围**: 1-10000
- **默认值**: 100
- **环境变量**: `SEARCH_MAX_LIMIT`

#### `search.enable_highlight`

是否启用搜索结果高亮。

- **类型**: Boolean
- **默认值**: true
- **环境变量**: `SEARCH_ENABLE_HIGHLIGHT`

#### `search.highlight_fragment_size`

高亮片段大小（字符数）。

- **类型**: Integer
- **范围**: 50-1000
- **默认值**: 200
- **环境变量**: `SEARCH_HIGHLIGHT_FRAGMENT_SIZE`

#### `search.enable_spell_check`

是否启用拼写检查。

- **类型**: Boolean
- **默认值**: false
- **环境变量**: `SEARCH_ENABLE_SPELL_CHECK`

#### `search.fuzzy_matching`

是否启用模糊匹配。

- **类型**: Boolean
- **默认值**: false
- **环境变量**: `SEARCH_FUZZY_MATCHING`

#### `search.fuzzy_distance`

模糊匹配的编辑距离。

- **类型**: Integer
- **范围**: 1-10
- **默认值**: 2
- **环境变量**: `SEARCH_FUZZY_DISTANCE`
- **说明**: 值越大，模糊匹配越宽松，但性能开销越大

---

## 环境变量优先级

当配置文件和环境变量同时存在时，**环境变量优先级更高**。

示例：

```bash
# 配置文件设置
# config.toml
[server]
address = "0.0.0.0:50051"

[bm25]
k1 = 1.2

# 环境变量覆盖
export SERVER_ADDRESS="0.0.0.0:8080"
export BM25_K1=1.5

# 实际使用值：
# server.address = "0.0.0.0:8080"  (环境变量优先)
# bm25.k1 = 1.5  (环境变量优先)
```

---

## 配置验证规则

BM25 服务在启动时会自动验证配置的有效性：

| 配置项 | 验证规则 |
|--------|----------|
| `bm25.k1` | 必须 >= 0.0 |
| `bm25.b` | 必须在 [0.0, 1.0] 范围内 |
| `bm25.avg_doc_length` | 必须 > 0.0 |
| `index.manager.writer_memory_budget` | 必须 >= 1MB |
| `search.default_limit` | 必须 > 0 |
| `search.max_limit` | 必须 > 0 |
| `search.default_limit` | 必须 <= `search.max_limit` |

---

## 最佳实践

### 1. 开发环境配置

```toml
[server]
address = "127.0.0.1:50051"

[storage.tantivy]
writer_memory_mb = 50

[index.manager]
writer_memory_budget = 50000000
merge_policy = "log"

[bm25]
k1 = 1.2
b = 0.75

[search]
default_limit = 10
enable_highlight = true
```

### 2. 生产环境配置

```toml
[server]
address = "0.0.0.0:50051"

[storage.tantivy]
writer_memory_mb = 200

[index.manager]
writer_memory_budget = 200000000
writer_num_threads = 0  # 自动检测 CPU
reader_cache_enabled = true
reader_reload_policy = "on_commit_with_delay"
merge_policy = "log"

[index.manager.log_merge_policy]
min_num_segments = 8
max_docs_before_merge = 10000000

[bm25]
k1 = 1.5
b = 0.8

[search]
default_limit = 20
max_limit = 200
enable_highlight = true
```

### 3. 高性能搜索配置

```toml
[storage.tantivy]
writer_memory_mb = 500

[index.manager]
writer_memory_budget = 500000000
writer_num_threads = 8
reader_cache_enabled = true

[bm25]
k1 = 2.0
b = 0.85
avg_doc_length = 200.0

[bm25.field_weights]
title = 3.0
content = 1.5

[search]
default_limit = 50
max_limit = 500
```

---

**文档版本**: 1.0  
**最后更新**: 2026-04-07
