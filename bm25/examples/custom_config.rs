use bm25_service::{Bm25Index, IndexManagerConfig, Result};

fn main() -> Result<()> {
    let config = IndexManagerConfig::builder()
        .writer_memory_mb(50)
        .writer_threads(2)
        .reader_cache(true)
        .build();

    let index = Bm25Index::create_with_config("/tmp/bm25_custom_config", config)?;

    index.add_document("1", "高性能搜索", "使用 BM25 算法实现高性能全文搜索")?;
    index.add_document("2", "可配置性", "支持灵活的配置选项，适应不同场景")?;

    let results = index.search("BM25", 10)?;
    for result in results {
        println!("{} - 得分：{:.4}", result.title.unwrap(), result.score);
    }

    Ok(())
}
