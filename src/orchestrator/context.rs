//! Recursive Project Context Discovery
//! 
//! Walks up the directory tree to discover AGENTS.md or CLAUDE.md files
//! and aggregates them into a comprehensive project context.

use std::path::{Path, PathBuf};
use anyhow::Result;
use tokio::fs;
use tracing::{info, debug};

pub struct ContextLoader;

impl ContextLoader {
    /// Discovers and aggregates project context files from the current directory upwards.
    pub async fn load_project_context() -> Result<String> {
        let cwd = std::env::current_dir()?;
        info!("Starting recursive context discovery from {:?}", cwd);
        
        let mut context_files = Vec::new();
        let mut current_dir = Some(cwd.as_path());

        while let Some(dir) = current_dir {
            if let Some(file) = Self::find_context_file(dir).await? {
                debug!("Found context file: {:?}", file);
                context_files.push(file);
            }
            current_dir = dir.parent();
        }

        // Aggregate contents (top-most parent first)
        let mut aggregated_content = String::new();
        for file_path in context_files.into_iter().rev() {
            let content = fs::read_to_string(&file_path).await?;
            aggregated_content.push_str(&format!("\n--- Context from {:?} ---\n", file_path));
            aggregated_content.push_str(&content);
            aggregated_content.push_str("\n");
        }

        Ok(aggregated_content)
    }

    async fn find_context_file(dir: &Path) -> Result<Option<PathBuf>> {
        let candidates = ["AGENTS.md", "CLAUDE.md", ".cursorrules", ".windsurfrules"];
        for candidate in candidates {
            let path = dir.join(candidate);
            if path.exists() {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_recursive_discovery() -> Result<()> {
        let root = tempdir()?;
        let sub = root.path().join("sub");
        fs::create_dir(&sub).await?;
        
        let root_file = root.path().join("AGENTS.md");
        let mut f1 = File::create(&root_file).await?;
        f1.write_all(b"Root Context").await?;
        
        let sub_file = sub.join("CLAUDE.md");
        let mut f2 = File::create(&sub_file).await?;
        f2.write_all(b"Sub Context").await?;

        // Change directory to sub for testing
        let original_cwd = std::env::current_dir()?;
        std::env::set_current_dir(&sub)?;
        
        let context = ContextLoader::load_project_context().await?;
        
        // Cleanup CWD before assertions
        std::env::set_current_dir(original_cwd)?;

        assert!(context.contains("Root Context"));
        assert!(context.contains("Sub Context"));
        
        Ok(())
    }
}
