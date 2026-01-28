use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const DEFAULT_CONFIG_NAME: &str = "paperclip.config.json";

/// Paperclip configuration file format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Source directory containing .pc files
    #[serde(default = "default_src_dir")]
    pub src_dir: String,

    /// Module directories for imports
    #[serde(default)]
    pub module_dirs: Vec<String>,

    /// Compiler output options
    #[serde(default)]
    pub compiler_options: Vec<CompilerOption>,
}

fn default_src_dir() -> String {
    "src".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerOption {
    /// Output formats to emit (e.g., "react", "html", "css")
    pub emit: Vec<String>,

    /// Optional output directory
    #[serde(rename = "outDir", skip_serializing_if = "Option::is_none")]
    pub out_dir: Option<String>,
}

impl Config {
    /// Load config from a directory
    pub fn load(cwd: &str) -> anyhow::Result<Self> {
        let config_path = PathBuf::from(cwd).join(DEFAULT_CONFIG_NAME);

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            // Return default config if none exists
            Ok(Config::default())
        }
    }

    /// Get absolute path to source directory
    pub fn get_src_dir(&self, cwd: &str) -> PathBuf {
        PathBuf::from(cwd).join(&self.src_dir)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            src_dir: default_src_dir(),
            module_dirs: vec![],
            compiler_options: vec![CompilerOption {
                emit: vec!["react".to_string()],
                out_dir: None,
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let json = r#"{
            "srcDir": "components",
            "moduleDirs": ["node_modules"],
            "compilerOptions": [
                { "emit": ["react", "css"], "outDir": "dist" }
            ]
        }"#;

        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.src_dir, "components");
        assert_eq!(config.module_dirs, vec!["node_modules"]);
        assert_eq!(config.compiler_options.len(), 1);
        assert_eq!(config.compiler_options[0].emit, vec!["react", "css"]);
        assert_eq!(config.compiler_options[0].out_dir, Some("dist".to_string()));
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.src_dir, "src");
        assert_eq!(config.module_dirs.len(), 0);
        assert_eq!(config.compiler_options[0].emit, vec!["react"]);
    }
}
