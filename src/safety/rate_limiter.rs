//! Rate Limiter
//! 
//! Prevents abuse by limiting operation frequency.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Simple rate limiter using token bucket algorithm
pub struct RateLimiter {
    /// Buckets for different operation types
    buckets: HashMap<String, TokenBucket>,
}

struct TokenBucket {
    tokens: u32,
    max_tokens: u32,
    last_refill: Instant,
    refill_rate: Duration,
}

impl TokenBucket {
    fn new(max_tokens: u32, refill_rate_secs: u64) -> Self {
        Self {
            tokens: max_tokens,
            max_tokens,
            last_refill: Instant::now(),
            refill_rate: Duration::from_secs(refill_rate_secs),
        }
    }

    fn try_consume(&mut self) -> bool {
        self.refill();
        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let elapsed = self.last_refill.elapsed();
        let refills = (elapsed.as_secs_f64() / self.refill_rate.as_secs_f64()) as u32;
        
        if refills > 0 {
            self.tokens = (self.tokens + refills).min(self.max_tokens);
            self.last_refill = Instant::now();
        }
    }

    fn reset(&mut self) {
        self.tokens = self.max_tokens;
        self.last_refill = Instant::now();
    }
}

impl RateLimiter {
    pub fn new() -> Self {
        let mut buckets = HashMap::new();
        
        // Configure limits for different operations
        buckets.insert("web_search".to_string(), TokenBucket::new(10, 60)); // 10 per minute
        buckets.insert("code_exec".to_string(), TokenBucket::new(5, 60));   // 5 per minute
        buckets.insert("llm_call".to_string(), TokenBucket::new(30, 60));   // 30 per minute
        
        Self {
            buckets,
        }
    }

    /// Check if a tool operation is allowed
    pub fn check_tool(&mut self, tool_name: &str) -> bool {
        if let Some(bucket) = self.buckets.get_mut(tool_name) {
            bucket.try_consume()
        } else {
            true // Default allow if not configured
        }
    }

    /// Check web search rate limit
    pub fn check_web_search(&mut self) -> bool {
        self.check_tool("web_search")
    }

    /// Reset all rate limiters
    pub fn reset(&mut self) {
        for bucket in self.buckets.values_mut() {
            bucket.reset();
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket() {
        let mut bucket = TokenBucket::new(3, 1);
        
        assert!(bucket.try_consume());
        assert!(bucket.try_consume());
        assert!(bucket.try_consume());
        assert!(!bucket.try_consume()); // Exhausted
    }

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new();
        
        // Should allow web searches up to limit
        for _ in 0..10 {
            assert!(limiter.check_web_search());
        }
        // 11th should fail
        assert!(!limiter.check_web_search());
    }
}
