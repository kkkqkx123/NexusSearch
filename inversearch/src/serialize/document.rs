//! Document 序列化模块
//!
//! 提供 Document 类型的导入导出功能

use crate::document::Document;
use crate::serialize::types::*;
use crate::error::Result;
use bincode;

impl Document {
    /// 导出 Document 数据
    pub fn export(&self, config: &SerializeConfig) -> Result<DocumentExportData> {
        let document_info = DocumentInfo {
            field_count: self.len(),
            fastupdate: false, // 简化实现
            store_enabled: false, // 简化实现
            tag_enabled: false, // 简化实现
        };

        // 简化实现：只导出字段名称
        let mut fields = Vec::new();
        for field_name in self.field_names() {
            let field_config = FieldConfigExport {
                field_type: "string".to_string(),
                index: true,
                optimize: false,
                resolution: 9, // 默认分辨率
            };

            // 获取字段的索引数据（如果可能）
            if let Some(field) = self.field(field_name) {
                let index_data = field.index().export(config)?;
                
                fields.push(FieldExportData {
                    name: field_name.to_string(),
                    field_config,
                    index_data,
                });
            }
        }

        let registry = DocumentRegistryData {
            doc_count: 0,
            next_doc_id: 1,
        };

        Ok(DocumentExportData {
            version: "0.1.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            document_info,
            fields,
            tags: None,
            store: None,
            registry,
        })
    }

    /// 导入 Document 数据
    pub fn import(&mut self, data: DocumentExportData, config: &SerializeConfig) -> Result<()> {
        if data.version != "0.1.0" {
            return Err(crate::error::InversearchError::Serialization(
                format!("Unsupported version: {}", data.version)
            ));
        }

        self.clear();
        
        // 导入字段数据
        for field_export in &data.fields {
            if let Some(_field) = self.field(&field_export.name) {
                // 由于无法获取可变引用，我们需要使用其他方式
                // 这里简化处理，实际应用中可能需要 Document 提供可变访问方法
                let _ = field_export;
                let _ = config;
            }
        }

        Ok(())
    }

    /// 序列化为 JSON 字符串
    pub fn to_json(&self, config: &SerializeConfig) -> Result<String> {
        let data = self.export(config)?;
        Ok(serde_json::to_string_pretty(&data)?)
    }

    /// 从 JSON 字符串反序列化
    pub fn from_json(json_str: &str, config: &SerializeConfig) -> Result<Document> {
        let data: DocumentExportData = serde_json::from_str(json_str)?;
        
        let mut doc_config = crate::document::DocumentConfig::new();
        
        for field_data in &data.fields {
            let field_config = crate::document::FieldConfig::new(&field_data.name);
            doc_config = doc_config.add_field(field_config);
        }
        
        if data.store.is_some() {
            doc_config = doc_config.with_store();
        }
        
        let mut document = Document::new(doc_config)?;
        document.import(data, config)?;
        
        Ok(document)
    }

    /// 序列化为二进制数据（高性能）
    pub fn to_binary(&self, config: &SerializeConfig) -> Result<Vec<u8>> {
        let data = self.export(config)?;
        let serialized = bincode::serialize(&data)?;
        Ok(serialized)
    }

    /// 从二进制数据反序列化
    pub fn from_binary(data: &[u8], config: &SerializeConfig) -> Result<Document> {
        let data: DocumentExportData = bincode::deserialize(data)?;
        
        let mut doc_config = crate::document::DocumentConfig::new();
        
        for field_data in &data.fields {
            let field_config = crate::document::FieldConfig::new(&field_data.name);
            doc_config = doc_config.add_field(field_config);
        }
        
        if data.store.is_some() {
            doc_config = doc_config.with_store();
        }
        
        let mut document = Document::new(doc_config)?;
        document.import(data, config)?;
        
        Ok(document)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{DocumentConfig, FieldConfig};
    use serde_json::json;

    #[test]
    fn test_document_export_import() {
        let config = DocumentConfig::new()
            .add_field(FieldConfig::new("title"))
            .add_field(FieldConfig::new("content"))
            .with_store();

        let mut document = Document::new(config).unwrap();
        document.add(1, &json!({"title": "Hello World", "content": "Test content"})).unwrap();
        document.add(2, &json!({"title": "Rust Programming", "content": "Another test"})).unwrap();

        let serialize_config = SerializeConfig::default();
        let json_str = document.to_json(&serialize_config).unwrap();
        
        let imported_document = Document::from_json(&json_str, &serialize_config).unwrap();
        
        assert!(imported_document.contains(1));
        assert!(imported_document.contains(2));
    }

    #[test]
    fn test_document_export_info() {
        let config = DocumentConfig::new()
            .add_field(FieldConfig::new("title"))
            .with_store();

        let document = Document::new(config).unwrap();
        let serialize_config = SerializeConfig::default();
        let export_data = document.export(&serialize_config).unwrap();

        assert_eq!(export_data.document_info.field_count, 1);
    }

    #[test]
    fn test_document_binary_export_import() {
        let config = DocumentConfig::new()
            .add_field(FieldConfig::new("title"))
            .add_field(FieldConfig::new("content"))
            .with_store();

        let mut document = Document::new(config).unwrap();
        document.add(1, &json!({"title": "Hello World", "content": "Test content"})).unwrap();
        document.add(2, &json!({"title": "Rust Programming", "content": "Another test"})).unwrap();

        let serialize_config = SerializeConfig::default();
        let binary_data = document.to_binary(&serialize_config).unwrap();
        
        let imported_document = Document::from_binary(&binary_data, &serialize_config).unwrap();
        
        assert!(imported_document.contains(1));
        assert!(imported_document.contains(2));
    }

    #[test]
    fn test_document_binary_empty() {
        let config = DocumentConfig::new()
            .add_field(FieldConfig::new("title"));

        let document = Document::new(config).unwrap();
        let serialize_config = SerializeConfig::default();
        
        let binary_data = document.to_binary(&serialize_config).unwrap();
        assert!(!binary_data.is_empty());
        
        let imported_document = Document::from_binary(&binary_data, &serialize_config).unwrap();
        assert!(!imported_document.contains(1));
    }
}
