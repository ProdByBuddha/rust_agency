//! LLM Response Cache
//! 
//! Provides a simple in-memory cache for LLM responses to avoid redundant computations.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use sha2::{Sha256, Digest};
use async_trait::async_trait;
use crate::agent::LLMProvider;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    model: String,
    prompt_hash: [u8; 32],
    system_hash: [u8; 32],
}

/// A cache for LLM responses
pub struct LLMCache {
    responses: Arc<RwLock<HashMap<CacheKey, String>>>,
}

impl LLMCache {
    pub fn new() -> Self {
        Self {
            responses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn hash(text: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        hasher.finalize().into()
    }

    pub async fn get(&self, model: &str, prompt: &str, system: Option<&str>) -> Option<String> {
        let key = CacheKey {
            model: model.to_string(),
            prompt_hash: Self::hash(prompt),
            system_hash: Self::hash(system.unwrap_or("")),
        };
        
        let responses = self.responses.read().await;
        responses.get(&key).cloned()
    }

    pub async fn set(&self, model: &str, prompt: &str, system: Option<&str>, response: String) {
        let key = CacheKey {
            model: model.to_string(),
            prompt_hash: Self::hash(prompt),
            system_hash: Self::hash(system.unwrap_or("")),
        };
        
        let mut responses = self.responses.write().await;
        responses.insert(key, response);
    }

    pub async fn clear(&self) {
        let mut responses = self.responses.write().await;
        responses.clear();
    }
}

impl Default for LLMCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Provider that wraps another provider with a cache
pub struct CachedProvider {
    inner: Arc<dyn LLMProvider>,
    cache: Arc<LLMCache>,
}

impl CachedProvider {
    pub fn new(inner: Arc<dyn LLMProvider>, cache: Arc<LLMCache>) -> Self {
        Self { inner, cache }
    }
}

#[async_trait]
impl LLMProvider for CachedProvider {
    async fn generate(&self, model: &str, prompt: String, system: Option<String>) -> anyhow::Result<String> {
        if let Some(cached) = self.cache.get(model, &prompt, system.as_deref()).await {
            tracing::debug!("LLM Cache Hit for model {}", model);
            return Ok(cached);
        }

        let response = self.inner.generate(model, prompt.clone(), system.clone()).await?;
        self.cache.set(model, &prompt, system.as_deref(), response.clone()).await;
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_get_set() {
        let cache = LLMCache::new();
        let model = "test-model";
        let prompt = "hello";
        let system = Some("be helpful");
        let response = "hi there".to_string();
        
        cache.set(model, prompt, system, response.clone()).await;
        
        let cached = cache.get(model, prompt, system).await;
        assert_eq!(cached.unwrap(), response);
    }

    #[tokio::test]
    async fn test_cache_miss_different_prompt() {
        let cache = LLMCache::new();
        cache.set("m", "p1", None, "r1".into()).await;
        
        let cached = cache.get("m", "p2", None).await;
        assert!(cached.is_none());
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = LLMCache::new();
        cache.set("m", "p", None, "r".into()).await;
        cache.clear().await;
        
        let cached = cache.get("m", "p", None).await;
        assert!(cached.is_none());
    }
}