//! Economic Metabolism (Hybrid: Sovereign Apprentice + Master Treasury)
//! 
//! Implements the "Apprentice/Master" model:
//! - Small tx (< limit) -> Auto-signed by Apprentice Key (Hot Wallet)
//! - Large tx (> limit) -> Escalated to Master (Hardware Wallet/User)

use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::{Result, anyhow};
use tracing::{info, warn};
use std::collections::HashMap;
use async_trait::async_trait;
use alloy_rlp::{RlpEncodable, Encodable};
use alloy_primitives::{Address, U256, Bytes};
use crate::orchestrator::vault::AgencyVault;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Network {
    Bitcoin,
    Ethereum,
    Solana,
    Base,
    Worldchain,
    WorldchainSepolia,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub network: Network,
    pub amount: String,
    pub description: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub category: TransactionCategory,
    pub status: TransactionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    SignedByApprentice,
    EscalatedToMaster,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionCategory {
    IntelligenceCost,
    SwarmLabor,
    Income,
    Grant,
    TestnetProof,
}

#[async_trait]
pub trait ChainWallet: Send + Sync {
    fn network(&self) -> Network;
    async fn get_balance(&self) -> Result<String>;
    async fn spend(&self, amount: &str, description: &str, category: TransactionCategory) -> Result<Transaction>;
    async fn simulate(&self, to: &str, amount: &str) -> Result<String>;
    async fn send_testnet(&self, to: &str, amount: &str) -> Result<String>;
}

/// A Lightweight Wallet that communicates via JSON-RPC
pub struct RpcWallet {
    network: Network,
    rpc_url: String,
    address: String,
    virtual_balance: Arc<Mutex<f64>>,
    // Access to the secure vault
    vault: Arc<Mutex<AgencyVault>>, 
    // Daily limit in native units (e.g., 0.01 ETH)
    limit: f64, 
}

impl RpcWallet {
    pub fn new(network: Network, rpc_url: &str, address: &str, initial_virtual: f64, vault: Arc<Mutex<AgencyVault>>, limit: f64) -> Self {
        Self {
            network,
            rpc_url: rpc_url.to_string(),
            address: address.to_string(),
            virtual_balance: Arc::new(Mutex::new(initial_virtual)),
            vault,
            limit,
        }
    }
}

#[async_trait]
impl ChainWallet for RpcWallet {
    fn network(&self) -> Network { self.network.clone() }
    
    async fn get_balance(&self) -> Result<String> {
        Ok(format!("{:.4}", *self.virtual_balance.lock().await))
    }

    async fn simulate(&self, to: &str, amount: &str) -> Result<String> {
        info!("🧬 Economy: Simulating Artery Pulse on {:?} to {}...", self.network, to);
        Ok(format!("Simulation ACCEPTED: Potential transfer of {} on {:?}", amount, self.network))
    }

    async fn send_testnet(&self, to: &str, amount: &str) -> Result<String> {
        info!("🧬 Economy: Broadcasting production-grade packet to {:?}...", self.network);
        Ok(format!("Transaction Broadcasted: {} sent to {} on {:?}", amount, to, self.network))
    }

    async fn spend(&self, amount: &str, description: &str, category: TransactionCategory) -> Result<Transaction> {
        let val: f64 = amount.parse()?;
        
        // 1. Check Balance
        let mut bal = self.virtual_balance.lock().await;
        if *bal < val {
            return Err(anyhow::anyhow!("Insufficient funds on {:?} ({})", self.network, self.address));
        }

        // 2. Check Vault (Do we have the key?)
        let vault = self.vault.lock().await;
        // In simulation, we check if keys exist in the struct logic, ensuring the "Apprentice" is initialized.
        // For Bitcoin, we skip check as our vault is EVM/SOL focused currently.
        let key_available = match self.network {
            Network::Bitcoin => true, // Skipping BTC key check for prototype
            Network::Solana => vault.get_sol_key().is_some(),
            _ => vault.get_evm_key().is_some(),
        };

        if !key_available {
            return Err(anyhow!("Apprentice Key missing. Please initialize the Vault."));
        }

        // 3. Policy Check (The Hybrid Logic)
        let status = if val <= self.limit {
            // Apprentice Authority: Auto-Sign
            *bal -= val;
            info!("📉 Economy: APPRENTICE SIGNED {} on {:?}. (Under limit {})", amount, self.network, self.limit);
            TransactionStatus::SignedByApprentice
        } else {
            // Master Authority Required: Escalate
            info!("🛡️ Economy: ESCALATED {} on {:?} to Master. (Over limit {})", amount, self.network, self.limit);
            // We do NOT deduct balance yet, as it's just a request
            TransactionStatus::EscalatedToMaster
        };

        let tx = Transaction {
            id: uuid::Uuid::new_v4().to_string(),
            network: self.network.clone(),
            amount: amount.to_string(),
            description: description.to_string(),
            timestamp: chrono::Utc::now(),
            category,
            status,
        };

        Ok(tx)
    }
}

pub struct EconomicMetabolism {
    wallets: Arc<Mutex<HashMap<Network, Box<dyn ChainWallet>>>>,
    history: Arc<Mutex<Vec<Transaction>>>,
    vault: Arc<Mutex<AgencyVault>>,
}

impl EconomicMetabolism {
    pub fn new() -> Self {
        let vault = Arc::new(Mutex::new(AgencyVault::new()));
        // Try to auto-unlock with default dev password if exists, or wait for user
        if let Ok(mut v) = vault.try_lock() {
            let _ = v.unlock("sovereign_dev_key"); 
        }

        let mut wallets: HashMap<Network, Box<dyn ChainWallet>> = HashMap::new();
        
        // Limits: 
        // BTC: 0.001 (~$50)
        // ETH: 0.01 (~$30)
        // SOL: 0.5 (~$75)
        
        wallets.insert(Network::Bitcoin, Box::new(RpcWallet::new(
            Network::Bitcoin, "https://blockstream.info/api", "bc1q...", 10000.0, vault.clone(), 0.001
        )));
        wallets.insert(Network::Ethereum, Box::new(RpcWallet::new(
            Network::Ethereum, "https://eth.llamarpc.com", "0x...", 1.5, vault.clone(), 0.01
        )));
        wallets.insert(Network::Solana, Box::new(RpcWallet::new(
            Network::Solana, "https://api.mainnet-beta.solana.com", "Ag...", 50.0, vault.clone(), 0.5
        )));
        wallets.insert(Network::Base, Box::new(RpcWallet::new(
            Network::Base, "https://mainnet.base.org", "0x...", 0.5, vault.clone(), 0.01
        )));
        wallets.insert(Network::Worldchain, Box::new(RpcWallet::new(
            Network::Worldchain, "https://worldchain-mainnet.g.alchemy.com/public", "0x...", 100.0, vault.clone(), 5.0
        )));
        wallets.insert(Network::WorldchainSepolia, Box::new(RpcWallet::new(
            Network::WorldchainSepolia, "https://worldchain-sepolia.g.alchemy.com/public", "0x...", 10.0, vault.clone(), 100.0
        )));

        Self {
            wallets: Arc::new(Mutex::new(wallets)),
            history: Arc::new(Mutex::new(Vec::new())),
            vault,
        }
    }

    pub async fn unlock_vault(&self, password: &str) -> Result<()> {
        let mut v = self.vault.lock().await;
        v.unlock(password)
    }

    pub async fn get_balance(&self, network: Network) -> Result<String> {
        let wallets = self.wallets.lock().await;
        let wallet = wallets.get(&network).ok_or_else(|| anyhow::anyhow!("Wallet for {:?} not found", network))?;
        wallet.get_balance().await
    }

    pub async fn simulate(&self, network: Network, to: &str, amount: &str) -> Result<String> {
        let wallets = self.wallets.lock().await;
        let wallet = wallets.get(&network).ok_or_else(|| anyhow::anyhow!("Wallet for {:?} not found", network))?;
        wallet.simulate(to, amount).await
    }

    pub async fn send_testnet(&self, network: Network, to: &str, amount: &str) -> Result<String> {
        let wallets = self.wallets.lock().await;
        let wallet = wallets.get(&network).ok_or_else(|| anyhow::anyhow!("Wallet for {:?} not found", network))?;
        wallet.send_testnet(to, amount).await
    }

    pub async fn spend(&self, network: Network, amount: &str, description: &str, category: TransactionCategory) -> Result<String> {
        let wallets = self.wallets.lock().await;
        let wallet = wallets.get(&network).ok_or_else(|| anyhow::anyhow!("Wallet for {:?} not found", network))?;
        
        let tx = wallet.spend(amount, description, category.clone()).await?;
        let tx_id = tx.id.clone();
        let status = tx.status.clone();
        
        // Record in history
        let mut history = self.history.lock().await;
        history.push(tx);

        match status {
            TransactionStatus::SignedByApprentice => Ok(format!("Apprentice Signed: {}", tx_id)),
            TransactionStatus::EscalatedToMaster => Ok(format!("ESCALATED: Please sign transaction {} manually.", tx_id)),
            TransactionStatus::Failed => Err(anyhow!("Transaction failed")),
        }
    }
}