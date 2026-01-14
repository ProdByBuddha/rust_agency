//! Reinforcement Learning (RL) Module
//! 
//! Provides infrastructure for experience collection and policy optimization.

use serde::{Deserialize, Serialize};
use crate::agent::ReActStep;
use candle_core::{Tensor, Result};
use candle_nn::{Optimizer, VarMap, AdamW, ParamsAdamW};

/// A single experience trajectory for RL training
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    /// The original query
    pub query: String,
    /// The sequence of reasoning steps and actions taken
    pub steps: Vec<ReActStep>,
    /// The final answer produced
    pub answer: String,
    /// The total reward received (weighted sum of extrinsic and intrinsic)
    pub total_reward: f32,
    /// Extrinsic reward from the Reward Model (RM)
    pub extrinsic_reward: f32,
    /// Intrinsic reward from curiosity (Novelty/Diversity)
    pub intrinsic_reward: f32,
}

/// A buffer for collecting experiences during autonomous operation
pub struct ExperienceBuffer {
    pub experiences: Vec<Experience>,
    pub max_size: usize,
}

impl ExperienceBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            experiences: Vec::new(),
            max_size,
        }
    }

    pub fn record(&mut self, experience: Experience) {
        if self.experiences.len() >= self.max_size {
            self.experiences.remove(0);
        }
        self.experiences.push(experience);
    }

    pub fn pop_batch(&mut self, batch_size: usize) -> Vec<Experience> {
        let count = std::cmp::min(self.experiences.len(), batch_size);
        if count == 0 { return Vec::new(); }
        // Simple FIFO for now, could be improved to reservoir sampling
        self.experiences.drain(0..count).collect()
    }

    pub fn clear(&mut self) {
        self.experiences.clear();
    }

    pub fn format_for_training(&self) -> String {
        serde_json::to_string_pretty(&self.experiences).unwrap_or_default()
    }
}

/// Trait for policy optimizers (e.g., PPO, GRPO)
#[async_trait::async_trait]
pub trait PolicyOptimizer: Send + Sync {
    /// Update the model policy using a batch of experiences
    async fn update_policy(&self, experiences: &[Experience]) -> anyhow::Result<f32>;
}

/// Group Relative Policy Optimization (GRPO) Trainer
/// 
/// GRPO optimizes the policy by comparing outputs within a group, 
/// eliminating the need for a separate Value Model.
pub struct GRPOTrainer {
    pub beta: f32, // KL penalty coefficient
    pub optimizer: std::sync::Mutex<AdamW>,
}

impl GRPOTrainer {
    pub fn new(beta: f32, varmap: &VarMap, learning_rate: f64) -> Result<Self> {
        let params = ParamsAdamW {
            lr: learning_rate,
            ..Default::default()
        };
        let optimizer = AdamW::new(varmap.all_vars(), params)?;
        Ok(Self { 
            beta,
            optimizer: std::sync::Mutex::new(optimizer),
        })
    }

    /// Calculate Relative Advantage for a group of experiences (G.5)
    pub fn calculate_advantages(&self, rewards: &[f32]) -> Vec<f32> {
        if rewards.is_empty() { return Vec::new(); }
        
        let mean = rewards.iter().sum::<f32>() / rewards.len() as f32;
        let std = (rewards.iter().map(|&r| (r - mean).powi(2)).sum::<f32>() / rewards.len() as f32).sqrt().max(1e-8);
        
        rewards.iter().map(|&r| (r - mean) / std).collect()
    }

    /// Calculate the surrogate loss for GRPO (Phase 4)
    /// loss = -[min(ratio * adv, clip(ratio, 1-eps, 1+adv) * adv) - beta * KL]
    pub fn calculate_loss(
        &self, 
        log_probs: &Tensor, 
        ref_log_probs: &Tensor, 
        advantages: &Tensor
    ) -> Result<Tensor> {
        // 1. Policy Ratio
        let ratio = (log_probs - ref_log_probs)?.exp()?;
        
        // 2. Clipped objective (Simplified for implementation)
        let surrogate1 = ratio.broadcast_mul(advantages)?;
        
        // 3. KL Divergence Penalty (Approximation: exp(log_p - log_ref) - (log_p - log_ref) - 1)
        let log_ratio = (log_probs - ref_log_probs)?;
        let one = Tensor::new(1.0f32, log_probs.device())?;
        
        // (exp(log_ratio) - log_ratio) - 1
        let term1 = (log_ratio.exp()? - &log_ratio)?;
        let kl = term1.broadcast_sub(&one)?;
        
        let beta_kl = kl.affine(self.beta as f64, 0.0)?;
        let loss = (surrogate1 - beta_kl)?.neg()?.mean_all()?;
        Ok(loss)
    }

    pub fn step(&self, loss: &Tensor) -> Result<()> {
        let grads = loss.backward()?;
        let mut opt = self.optimizer.lock().unwrap();
        opt.step(&grads)?;
        Ok(())
    }
}

/// A mock optimizer for testing and logging
pub struct LoggingOptimizer;

#[async_trait::async_trait]
impl PolicyOptimizer for LoggingOptimizer {
    async fn update_policy(&self, experiences: &[Experience]) -> anyhow::Result<f32> {
        let avg_reward: f32 = experiences.iter().map(|e| e.total_reward).sum::<f32>() / experiences.len() as f32;
        tracing::info!("LoggingOptimizer: Processed {} experiences. Average Reward: {:.4}", experiences.len(), avg_reward);
        Ok(avg_reward)
    }
}
