use bm25_service::{Bm25Index, Result};

fn main() -> Result<()> {
    let index = Bm25Index::create("/tmp/bm25_example")?;

    index.add_document("1", "Rust 编程", "Rust 是一门系统编程语言，注重安全性和性能")?;
    index.add_document("2", "Java 编程", "Java 是一门面向对象编程语言，广泛应用于企业开发")?;
    index.add_document("3", "Python 编程", "Python 是一门动态类型语言，适合快速开发和数据分析")?;

    let query = "Rust";
    let results = index.search(query, 10)?;

    println!("搜索 '{}', 找到 {} 个结果:\n", query, results.len());
    for (i, result) in results.iter().enumerate() {
        println!("{}. {} (得分：{:.4})", i + 1, result.title.as_ref().unwrap(), result.score);
        if let Some(content) = &result.content {
            println!("   {}", content);
        }
        println!();
    }

    println!("索引中文档总数：{}", index.count()?);

    Ok(())
}
