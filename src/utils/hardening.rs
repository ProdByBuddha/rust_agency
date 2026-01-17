//! System Hardening & Prerequisites
//! 
//! Performs critical environment checks before the agency starts.
//! Ensures that "Sharp Edges" (Permissions, Libraries, Profiles) are handled gracefully.

use anyhow::{Result, Context};
use std::path::Path;
use tracing::{info, warn, error};
use std::process::Command;

pub struct SystemHardening;

impl SystemHardening {
    /// Run all startup checks. Returns Err if a critical requirement is missing.
    pub async fn verify_environment() -> Result<()> {
        info!("🛡️  Hardening: Verifying system environment...");

        Self::check_sandbox_profile().await?;
        Self::check_macos_permissions().await; // Warning only
        Self::check_onnx_library().await;      // Warning only

        Ok(())
    }

    /// Critical: Ensure the Seatbelt profile exists
    async fn check_sandbox_profile() -> Result<()> {
        let profile_path = Path::new("conduit.sb");
        if !profile_path.exists() {
            // In a real release, we might write a default one here.
            // For now, we fail because safety is mandatory.
            return Err(anyhow::anyhow!("CRITICAL: Sandbox profile 'conduit.sb' not found. The Immune System cannot function."));
        }
        info!("✅ Hardening: Sandbox profile verified.");
        Ok(())
    }

    /// Warning: Check if we have Accessibility Permissions (for Hands)
    /// This is hard to check deterministically without triggering a prompt,
    /// but we can try a benign check.
    async fn check_macos_permissions() {
        if cfg!(target_os = "macos") {
            // We can't easily check "Is Accessibility Granted" via CLI without private APIs.
            // Instead, we verify if we are running in a terminal likely to have them,
            // or just warn the user.
            warn!("⚠️  Hardening: macOS detected. Ensure Terminal/App has 'Accessibility' permissions for the 'Hands' tool to function.");
        }
    }

    /// Warning: Check if ONNX runtime is present (for Stomach/Eyes)
    async fn check_onnx_library() {
        // Check for dylib in CWD or standard paths
        let local_path = Path::new("libonnxruntime.dylib");
        let env_path = std::env::var("ORT_DYLIB_PATH").ok();
        
        if local_path.exists() {
            info!("✅ Hardening: Found local ONNX Runtime library.");
        } else if let Some(p) = env_path {
            if Path::new(&p).exists() {
                info!("✅ Hardening: Found ONNX Runtime via ORT_DYLIB_PATH.");
            } else {
                warn!("⚠️  Hardening: ORT_DYLIB_PATH set but file missing: {}", p);
            }
        } else {
            warn!("⚠️  Hardening: ONNX Runtime library not found. 'Stomach' (Memory) and 'Eyes' (Vision) may fail or trigger downloads.");
        }
    }
}
