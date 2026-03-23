# Inversearch 服务文档索引

## 文档列表

本文档目录包含 Inversearch 服务从 JavaScript 实现到 Rust 实现的完整规划和分析。

### 核心文档

| 文档 | 说明 |
|------|------|
| [execution-plan.md](./execution-plan.md) | 分阶段执行方案，包含5个阶段的详细实施计划 |
| [technical-analysis.md](./technical-analysis.md) | 技术分析，详细说明从JS到Rust的转换要点 |
| [module-mapping.md](./module-mapping.md) | 模块映射对照表，提供完整的函数和数据结构映射 |

---

## 文档导航

### execution-plan.md

**适用场景**：需要了解整体实施计划和阶段划分时

**主要内容**：
- 5个实施阶段的详细说明
- 每个阶段的模块映射
- JS实现到Rust实现的详细对应关系
- 验收标准和依赖关系

**快速链接**：
- [阶段一：基础数据结构和工具模块](./execution-plan.md#阶段一基础数据结构和工具模块)
- [阶段二：编码器和分词模块](./execution-plan.md#阶段二编码器和分词模块)
- [阶段三：索引管理模块](./execution-plan.md#阶段三索引管理模块)
- [阶段四：搜索和结果处理模块](./execution-plan.md#阶段四搜索和结果处理模块)
- [阶段五：缓存、存储和集成](./execution-plan.md#阶段五缓存存储和集成)

### technical-analysis.md

**适用场景**：需要深入了解技术细节和实现要点时

**主要内容**：
- 数据结构映射（倒排索引、密钥存储、编码器配置）
- 算法转换（评分算法、交集算法、哈希算法）
- 性能优化（内存优化、查询优化、缓存优化）
- 错误处理和并发处理
- 测试策略和部署考虑

**快速链接**：
- [数据结构映射](./technical-analysis.md#一数据结构映射)
- [算法转换](./technical-analysis.md#二算法转换)
- [性能优化](./technical-analysis.md#三性能优化)
- [错误处理](./technical-analysis.md#四错误处理)
- [并发处理](./technical-analysis.md#五并发处理)

### module-mapping.md

**适用场景**：需要查找特定函数或数据结构的对应关系时

**主要内容**：
- 核心模块映射（索引、搜索、编码器、字符集）
- 工具模块映射（通用工具、密钥存储）
- 结果处理模块映射（交集、高亮、解析器）
- 存储和缓存模块映射
- 类型定义映射（索引选项、搜索选项、编码器选项）

**快速链接**：
- [索引模块映射](./module-mapping.md#11-索引模块)
- [搜索模块映射](./module-mapping.md#12-搜索模块)
- [编码器模块映射](./module-mapping.md#13-编码器模块)
- [字符集模块映射](./module-mapping.md#14-字符集模块)
- [类型定义映射](./module-mapping.md#五类型定义映射)

---

## 使用指南

### 开始实施前

1. 阅读 [execution-plan.md](./execution-plan.md) 了解整体计划
2. 查看 [technical-analysis.md](./technical-analysis.md) 了解技术要点
3. 参考 [module-mapping.md](./module-mapping.md) 查找具体映射

### 实施过程中

1. 按阶段顺序执行，参考 [execution-plan.md](./execution-plan.md)
2. 遇到技术问题时，查阅 [technical-analysis.md](./technical-analysis.md)
3. 需要查找函数对应关系时，使用 [module-mapping.md](./module-mapping.md)

### 验收和测试

1. 每个阶段完成后，对照 [execution-plan.md](./execution-plan.md) 的验收标准
2. 参考 [technical-analysis.md](./technical-analysis.md) 的测试策略
3. 使用 [module-mapping.md](./module-mapping.md) 确保功能一致性

---

## JS 实现文件索引

| JS 文件 | 说明 | 对应 Rust 模块 |
|---------|------|---------------|
| `src/index.js` | 索引核心结构 | `src/index/mod.rs` |
| `src/index/add.js` | 索引构建器 | `src/index/builder.rs` |
| `src/index/remove.js` | 索引删除器 | `src/index/remover.rs` |
| `src/index/search.js` | 搜索核心逻辑 | `src/search/mod.rs` |
| `src/encoder.js` | 编码器核心逻辑 | `src/encoder/mod.rs` |
| `src/charset/` | 字符集规范化 | `src/charset/mod.rs` |
| `src/cache.js` | 缓存管理 | `src/cache/mod.rs` |
| `src/keystore.js` | 密钥存储优化 | `src/keystore/mod.rs` |
| `src/intersect.js` | 交集和并集计算 | `src/intersect/mod.rs` |
| `src/resolver.js` | 结果解析器 | `src/resolver/mod.rs` |
| `src/document/highlight.js` | 结果高亮 | `src/highlight/mod.rs` |
| `src/db/redis/index.js` | Redis 存储 | `src/storage/mod.rs` |
| `src/common.js` | 通用工具函数 | `src/common/mod.rs` |
| `src/type.js` | 类型定义 | `src/type/mod.rs` |
| `src/config.js` | 配置管理 | `src/config/mod.rs` |

---

## 关键概念

### 倒排索引

用于快速搜索的数据结构，将词映射到包含该词的文档列表。

**JS 实现**：`src/index.js` 的 `this.map`
**Rust 实现**：`src/index/mod.rs` 的 `map: HashMap<String, Vec<Vec<DocId>>>`

### 上下文搜索

基于词序的搜索，支持查找连续的词组。

**JS 实现**：`src/index.js` 的 `this.ctx`
**Rust 实现**：`src/index/mod.rs` 的 `ctx: HashMap<String, HashMap<String, Vec<Vec<DocId>>>>`

### 评分机制

根据词在文档中的位置和文档长度计算相关性得分。

**JS 实现**：`src/index/add.js` 的 `get_score` 函数
**Rust 实现**：`src/index/builder.rs` 的 `get_score` 函数

### 编码器

处理文本预处理、分词、规范化等。

**JS 实现**：`src/encoder.js`
**Rust 实现**：`src/encoder/mod.rs`

### 密钥存储

优化的哈希表实现，支持大容量存储。

**JS 实现**：`src/keystore.js`
**Rust 实现**：`src/keystore/mod.rs`

---

## 常见问题

### Q1: 如何开始实施？

A: 按照 [execution-plan.md](./execution-plan.md) 的阶段顺序，从阶段一开始实施。

### Q2: 如何确保 Rust 实现与 JS 实现一致？

A: 参考 [module-mapping.md](./module-mapping.md) 的详细映射表，确保每个函数和数据结构都对应。

### Q3: 如何处理性能优化？

A: 参考 [technical-analysis.md](./technical-analysis.md) 的性能优化章节，了解内存优化、查询优化等策略。

### Q4: 如何进行测试？

A: 参考 [technical-analysis.md](./technical-analysis.md) 的测试策略章节，包括单元测试、集成测试和性能测试。

### Q5: 如何处理错误和并发？

A: 参考 [technical-analysis.md](./technical-analysis.md) 的错误处理和并发处理章节。

---

## 更新日志

| 日期 | 版本 | 更新内容 |
|------|------|---------|
| 2026-02-04 | 1.0 | 初始版本，包含执行方案、技术分析和模块映射 |

---

## 联系方式

如有问题或建议，请通过项目仓库的 Issue 系统反馈。
