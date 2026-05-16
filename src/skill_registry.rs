use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillConfig {
    pub skill: SkillMeta,
    pub tool: Option<ToolDef>,
    pub implementation: ImplConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMeta {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub trigger_keywords: Vec<String>,
    pub complexity: String,
    #[serde(default)]
    pub requires_confirmation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplConfig {
    #[serde(rename = "type")]
    pub impl_type: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub auth_env: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub data_template: Option<String>,
    #[serde(default)]
    pub extras: Option<HashMap<String, String>>,
}

pub trait Skill: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn to_tool_schema(&self) -> serde_json::Value;
}

pub struct TomlSkill {
    config: SkillConfig,
    instructions: Option<String>,
}

impl TomlSkill {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: SkillConfig = toml::from_str(&content)?;
        let md_path = path.with_extension("md");
        let instructions = if md_path.exists() {
            Some(std::fs::read_to_string(&md_path)?)
        } else {
            None
        };
        Ok(Self {
            config,
            instructions,
        })
    }

    pub fn instructions(&self) -> Option<&str> {
        self.instructions.as_deref()
    }
}

impl Skill for TomlSkill {
    fn name(&self) -> &str {
        &self.config.skill.name
    }

    fn description(&self) -> &str {
        &self.config.skill.description
    }

    fn to_tool_schema(&self) -> serde_json::Value {
        let tool = self.config.tool.as_ref();
        let params = tool.map(|t| {
            let props: serde_json::Value = t
                .parameters
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        serde_json::json!({
                            "type": v,
                            "description": format!("The {} parameter", k),
                        }),
                    )
                })
                .collect();
            serde_json::json!({
                "type": "object",
                "properties": props,
                "required": t.parameters.keys().collect::<Vec<_>>(),
            })
        });
        serde_json::json!({
            "type": "function",
            "function": {
                "name": tool.map(|t| t.name.as_str()).unwrap_or(self.name()),
                "description": self.description(),
                "parameters": params,
            }
        })
    }
}

pub struct SkillRegistry {
    skills: HashMap<String, Box<dyn Skill>>,
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    pub fn register(&mut self, skill: Box<dyn Skill>) {
        self.skills.insert(skill.name().to_string(), skill);
    }

    pub fn load_from_dir(&mut self, dir: &Path) -> Result<usize, Box<dyn std::error::Error>> {
        let mut count = 0;
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "toml") {
                match TomlSkill::load(&path) {
                    Ok(skill) => {
                        self.register(Box::new(skill));
                        count += 1;
                    }
                    Err(e) => eprintln!("Failed to load skill {:?}: {}", path, e),
                }
            }
        }
        Ok(count)
    }

    pub fn tools_for_prompt(&self) -> Vec<serde_json::Value> {
        self.skills.values().map(|s| s.to_tool_schema()).collect()
    }

    pub fn get(&self, name: &str) -> Option<&dyn Skill> {
        self.skills.get(name).map(|s| s.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_load_toml_skill() {
        let toml_str = r#"
[skill]
name = "web_search"
description = "Search the web"
complexity = "Standard"

[tool]
name = "web_search"
parameters = { query = "string", max_results = "integer" }

[implementation]
type = "http"
url = "http://127.0.0.1:8080/search"
"#;
        let config: SkillConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.skill.name, "web_search");
        assert_eq!(config.implementation.impl_type, "http");
    }

    #[test]
    fn test_registry_load_and_query() {
        let mut registry = SkillRegistry::new();
        let dir = std::env::temp_dir().join("agent_test_skills");
        std::fs::create_dir_all(&dir).unwrap();
        let skill_path = dir.join("test_skill.toml");
        let mut f = std::fs::File::create(&skill_path).unwrap();
        f.write_all(
            br#"
[skill]
name = "test_skill"
description = "A test skill"
complexity = "Trivial"

[implementation]
type = "http"
"#,
        )
        .unwrap();

        let count = registry.load_from_dir(&dir).unwrap();
        assert_eq!(count, 1);
        assert!(registry.get("test_skill").is_some());
        let tools = registry.tools_for_prompt();
        assert_eq!(tools.len(), 1);
        std::fs::remove_dir_all(&dir).ok();
    }
}
