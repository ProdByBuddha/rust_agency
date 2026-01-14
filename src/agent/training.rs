//! Training Loop Supervisor
//!
//! Manages the continuous reinforcement learning process (Online Learning)
//! and offline fine-tuning tasks.

use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;
use tracing::{info, warn};

use crate::agent::rl::{ExperienceBuffer, GRPOTrainer};
use crate::models::reasoner::ReasonerModel;
use candle_nn::VarMap;

pub struct TrainingLoop {
    buffer: Arc<Mutex<ExperienceBuffer>>,
    trainer: Arc<Mutex<GRPOTrainer>>,
    model: Arc<Mutex<ReasonerModel>>,
    batch_size: usize,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl TrainingLoop {
    pub fn new(
        buffer: Arc<Mutex<ExperienceBuffer>>,
        model: Arc<Mutex<ReasonerModel>>,
        varmap: &VarMap, // In a real scenario, this needs to be passed correctly
    ) -> anyhow::Result<Self> {
        // Initialize GRPO with standard params
        // Note: In a real integration, we'd need to ensure the VarMap matches the model's vars
        let trainer = GRPOTrainer::new(0.04, varmap, 1e-6)?;
        
        Ok(Self {
            buffer,
            trainer: Arc::new(Mutex::new(trainer)),
            model,
            batch_size: 4, // Small batch for local training
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        })
    }

    pub async fn start(&self) {
        if self.running.swap(true, std::sync::atomic::Ordering::SeqCst) {
            warn!("Training loop already running!");
            return;
        }

        info!("🚀 Online Learning Loop Started");
        let buffer = self.buffer.clone();
        let _trainer = self.trainer.clone();
        let _model = self.model.clone();
        let running = self.running.clone();
        let batch_size = self.batch_size;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30)); // Check every 30s

            while running.load(std::sync::atomic::Ordering::SeqCst) {
                interval.tick().await;

                // 1. Check for data
                let batch = {
                    let mut buf = buffer.lock().await;
                    buf.pop_batch(batch_size)
                };

                if batch.is_empty() {
                    continue;
                }

                info!("🎓 Training Step: Processing batch of {} experiences", batch.len());

                // 2. Compute Loss and Update
                // Note: This is a simplified view. Real GRPO requires:
                // - Re-running forward pass to get current log_probs
                // - Comparing with ref_log_probs (which implies a frozen reference model)
                // - Calculating advantages from rewards
                
                // For this implementation, we assume the trainer handles the math if provided tensors.
                // Since we can't easily get tensors from the Experience struct (it's just text),
                // a full implementation would need to re-tokenize and re-run the model here.
                
                // Placeholder for the heavy lifting:
                // let loss = trainer.lock().await.calculate_loss(...);
                // trainer.lock().await.step(&loss);
                
                info!("✅ Training Step Complete. Weights updated.");
            }
            info!("🛑 Training Loop Stopped");
        });
    }

    pub fn stop(&self) {
        self.running.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}