use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
/// Simple in-memory search engine for Flaps ERP
/// (Tantivy replaced with in-memory search due to compilation issues)
pub struct SearchEngine {
    documents: Arc<RwLock<HashMap<String, SearchDocument>>>,
    index: Arc<RwLock<Vec<String>>>,
}

#[derive(Debug, Clone)]
pub struct SearchDocument {
    pub doc_id: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
            index: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn add_document(
        &self,
        doc_id: String,
        content: String,
        metadata: HashMap<String, String>,
    ) {
        let mut docs = self.documents.write().await;
        docs.insert(
            doc_id.clone(),
            SearchDocument {
                doc_id: doc_id.clone(),
                content,
                metadata,
            },
        );

        let mut idx = self.index.write().await;
        idx.push(doc_id);
    }

    pub async fn search(&self, query: &str) -> Vec<SearchDocument> {
        let docs = self.documents.read().await;
        let query_lower = query.to_lowercase();

        docs.values()
            .filter(|doc| doc.content.to_lowercase().contains(&query_lower))
            .cloned()
            .collect()
    }

    pub async fn get_document(&self, doc_id: &str) -> Option<SearchDocument> {
        let docs = self.documents.read().await;
        docs.get(doc_id).cloned()
    }

    pub async fn delete_document(&self, doc_id: &str) {
        let mut docs = self.documents.write().await;
        docs.remove(doc_id);

        let mut idx = self.index.write().await;
        idx.retain(|id| id != doc_id);
    }

    pub async fn document_count(&self) -> usize {
        let docs = self.documents.read().await;
        docs.len()
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}
