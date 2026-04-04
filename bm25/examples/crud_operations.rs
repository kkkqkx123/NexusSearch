use bm25_service::{Bm25Index, Result};

fn main() -> Result<()> {
    let index = Bm25Index::create("/tmp/bm25_crud")?;

    index.add_document("1", "初始标题", "初始内容")?;
    println!("添加文档后总数：{}", index.count()?);

    index.update_document("1", "更新后的标题", "更新后的内容")?;
    println!("更新文档后总数：{}", index.count()?);

    let results = index.search("更新", 10)?;
    println!("搜索 '更新' 找到 {} 个结果", results.len());
    if let Some(result) = results.first() {
        println!("结果：{} - {}", result.title.as_ref().unwrap(), result.content.as_ref().unwrap());
    }

    index.delete_document("1")?;
    println!("删除文档后总数：{}", index.count()?);

    let results = index.search("更新", 10)?;
    println!("删除后搜索 '更新' 找到 {} 个结果", results.len());

    Ok(())
}
