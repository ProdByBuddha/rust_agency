//! Markdown-Based Skill Discovery
//! 
//! Allows the agency to discover capabilities by reading .md files
//! in a skills/ directory. Each file contains YAML frontmatter
//! with metadata and a body with instructions.

use anyhow::Context;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::agent::AgentResult;
use super::{Tool, ToolOutput};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct SkillMetadata {
    name: String,
    description: String,
    #[serde(default)]
    version: String,
}

pub struct MarkdownSkill {
    metadata: SkillMetadata,
    instructions: String,
    pub path: PathBuf,
}

impl MarkdownSkill {
    pub async fn load_from_file(path: &Path) -> anyhow::Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        
        // Extract frontmatter
        let parts: Vec<&str> = content.split("---").collect();
        if parts.len() < 3 {
            anyhow::bail!("Skill file {:?} missing YAML frontmatter delimited by ---", path);
        }
        
        let yaml_str = parts[1];
        let instructions = parts[2..].join("---").trim().to_string();
        
        let metadata: SkillMetadata = serde_yaml::from_str(yaml_str)
            .context(format!("Failed to parse YAML frontmatter in {:?}", path))?;
            
        Ok(Self {
            metadata,
            instructions,
            path: path.to_path_buf(),
        })
    }

    /// Promotes the skill from custom/experimental to standard
    pub async fn promote(&self, standard_dir: &Path) -> anyhow::Result<()> {
        if !standard_dir.exists() {
            tokio::fs::create_dir_all(standard_dir).await?;
        }
        
        let file_name = self.path.file_name().context("Invalid skill path")?;
        let new_path = standard_dir.join(file_name);
        
        info!("🎓 Promoting skill '{}' to standard set at {:?}", self.metadata.name, new_path);
        tokio::fs::rename(&self.path, &new_path).await?;
        Ok(())
    }
}

#[async_trait]
impl Tool for MarkdownSkill {
    fn name(&self) -> String {
        format!("skill__{}", self.metadata.name.to_lowercase().replace(" ", "_"))
    }

    fn description(&self) -> String {
        format!("{} (Skill). Instructions: {}", self.metadata.description, self.instructions)
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "Input for the skill"
                }
            },
            "required": ["input"]
        })
    }

    async fn execute(&self, params: Value) -> AgentResult<ToolOutput> {
        let input = params["input"].as_str().unwrap_or("");
        info!("Executing skill {}: input={}", self.name(), input);
        
        Ok(ToolOutput::success(
            json!({ "status": "skill_invoked", "skill": self.metadata.name }),
            format!("Skill '{}' invoked. Please follow these specialized instructions: \n\n{}", 
                self.metadata.name, self.instructions)
        ))
    }
}

pub struct SkillLoader;

impl SkillLoader {
    pub async fn discover_skills(dir_path: impl AsRef<Path>) -> anyhow::Result<Vec<MarkdownSkill>> {
        let path = dir_path.as_ref();
        if !path.exists() {
            return Ok(Vec::new());
        }

        let mut skills = Vec::new();
        let mut entries = tokio::fs::read_dir(path).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let p = entry.path();
            if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("md") {
                match MarkdownSkill::load_from_file(&p).await {
                    Ok(skill) => {
                        info!("Discovered skill: {} (v{})", skill.metadata.name, skill.metadata.version);
                        skills.push(skill);
                    },
                    Err(e) => warn!("Failed to load skill at {:?}: {}", p, e),
                }
            }
        }
        
        Ok(skills)
    }
}