//! 测试查询数据固件
//!
//! 提供各种测试场景所需的查询数据

/// 测试查询结构
#[derive(Debug, Clone)]
pub struct TestQuery {
    pub query: &'static str,
    pub expected_doc_ids: &'static [u64],
    pub description: &'static str,
}

/// 基本搜索查询
pub const BASIC_QUERIES: &[TestQuery] = &[
    TestQuery {
        query: "Rust",
        expected_doc_ids: &[1, 100, 200, 300, 400],
        description: "单关键词搜索 - 应该返回所有包含 Rust 的文档",
    },
    TestQuery {
        query: "programming language",
        expected_doc_ids: &[1, 2, 3, 4, 5, 6, 7, 8],
        description: "多关键词搜索 - 应该返回包含任一关键词的文档",
    },
    TestQuery {
        query: "systems programming",
        expected_doc_ids: &[1, 4, 7],
        description: "特定类型搜索",
    },
    TestQuery {
        query: "web",
        expected_doc_ids: &[3, 5],
        description: "Web 相关语言搜索",
    },
];

/// CJK 查询
pub const CJK_QUERIES: &[TestQuery] = &[
    TestQuery {
        query: "编程",
        expected_doc_ids: &[100, 101, 400],
        description: "中文关键词搜索",
    },
    TestQuery {
        query: "プログラミング",
        expected_doc_ids: &[200, 201, 202, 401],
        description: "日文关键词搜索",
    },
    TestQuery {
        query: "프로그래밍",
        expected_doc_ids: &[300, 301, 302],
        description: "韩文关键词搜索",
    },
    TestQuery {
        query: "语言",
        expected_doc_ids: &[100, 101, 102],
        description: "中文字搜索",
    },
];

/// 边界情况查询
pub const EDGE_CASE_QUERIES: &[TestQuery] = &[
    TestQuery {
        query: "",
        expected_doc_ids: &[],
        description: "空查询 - 应该返回空结果",
    },
    TestQuery {
        query: "xyz123nonexistent",
        expected_doc_ids: &[],
        description: "不存在的词 - 应该返回空结果",
    },
    TestQuery {
        query: "a",
        expected_doc_ids: &[],
        description: "单个字符 - 取决于最小长度配置",
    },
    TestQuery {
        query: "the",
        expected_doc_ids: &[],
        description: "停用词 - 应该被过滤",
    },
];

/// 模糊匹配查询
pub const FUZZY_QUERIES: &[TestQuery] = &[
    TestQuery {
        query: "Rst",
        expected_doc_ids: &[1, 100, 200, 300, 400],
        description: "拼写错误 - 应该匹配 Rust",
    },
    TestQuery {
        query: "progrmming",
        expected_doc_ids: &[1, 2, 3, 4, 5, 6, 7, 8],
        description: "拼写错误 - 应该匹配 programming",
    },
];
