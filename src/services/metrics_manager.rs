use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Default, Clone, Serialize)]
pub struct MetricsData {
    pub language_usage: HashMap<String, u64>,
    pub intent_usage: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
pub struct MetricsManager {
    inner: Arc<RwLock<MetricsData>>,
}

impl Default for MetricsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(MetricsData::default())),
        }
    }

    pub async fn increment_language(&self, lang: &str) {
        let mut data = self.inner.write().await;
        *data.language_usage.entry(lang.to_string()).or_insert(0) += 1;
    }

    pub async fn increment_intent(&self, intent: &str) {
        let mut data = self.inner.write().await;
        *data.intent_usage.entry(intent.to_string()).or_insert(0) += 1;
    }

    pub async fn get_metrics(&self) -> MetricsData {
        self.inner.read().await.clone()
    }
}
