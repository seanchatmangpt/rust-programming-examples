//! Configuration management for cloudctl

use crate::error::CloudError;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Application configuration.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Current project
    pub project: Option<String>,

    /// Default region
    pub region: Option<String>,

    /// Default zone
    pub zone: Option<String>,

    /// API endpoint
    pub endpoint: Option<String>,

    /// Output format preference
    pub output_format: Option<String>,

    /// Profiles
    #[serde(default)]
    pub profiles: std::collections::HashMap<String, Profile>,
}

/// A configuration profile.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Profile {
    /// Project for this profile
    pub project: Option<String>,

    /// Region for this profile
    pub region: Option<String>,

    /// Zone for this profile
    pub zone: Option<String>,

    /// Custom endpoint
    pub endpoint: Option<String>,
}

impl Config {
    /// Create a new empty config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the default config path.
    pub fn default_path() -> std::path::PathBuf {
        dirs_path().join("config.toml")
    }

    /// Get a value by key.
    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "project" => self.project.clone(),
            "region" => self.region.clone(),
            "zone" => self.zone.clone(),
            "endpoint" => self.endpoint.clone(),
            "output_format" => self.output_format.clone(),
            _ => None,
        }
    }

    /// Set a value by key.
    pub fn set(&mut self, key: &str, value: String) -> Result<(), CloudError> {
        match key {
            "project" => self.project = Some(value),
            "region" => self.region = Some(value),
            "zone" => self.zone = Some(value),
            "endpoint" => self.endpoint = Some(value),
            "output_format" => self.output_format = Some(value),
            _ => return Err(CloudError::InvalidArgument(format!("Unknown key: {}", key))),
        }
        Ok(())
    }

    /// Unset a value by key.
    pub fn unset(&mut self, key: &str) -> Result<(), CloudError> {
        match key {
            "project" => self.project = None,
            "region" => self.region = None,
            "zone" => self.zone = None,
            "endpoint" => self.endpoint = None,
            "output_format" => self.output_format = None,
            _ => return Err(CloudError::InvalidArgument(format!("Unknown key: {}", key))),
        }
        Ok(())
    }

    /// List all configuration values.
    pub fn list(&self) -> Vec<(String, String)> {
        let mut values = Vec::new();

        if let Some(v) = &self.project {
            values.push(("project".to_string(), v.clone()));
        }
        if let Some(v) = &self.region {
            values.push(("region".to_string(), v.clone()));
        }
        if let Some(v) = &self.zone {
            values.push(("zone".to_string(), v.clone()));
        }
        if let Some(v) = &self.endpoint {
            values.push(("endpoint".to_string(), v.clone()));
        }
        if let Some(v) = &self.output_format {
            values.push(("output_format".to_string(), v.clone()));
        }

        values
    }

    /// Merge with a profile.
    pub fn with_profile(&self, profile_name: &str) -> Result<Config, CloudError> {
        if let Some(profile) = self.profiles.get(profile_name) {
            let mut merged = self.clone();
            if profile.project.is_some() {
                merged.project = profile.project.clone();
            }
            if profile.region.is_some() {
                merged.region = profile.region.clone();
            }
            if profile.zone.is_some() {
                merged.zone = profile.zone.clone();
            }
            if profile.endpoint.is_some() {
                merged.endpoint = profile.endpoint.clone();
            }
            Ok(merged)
        } else {
            Err(CloudError::not_found("profile", profile_name))
        }
    }
}

impl Clone for Config {
    fn clone(&self) -> Self {
        Self {
            project: self.project.clone(),
            region: self.region.clone(),
            zone: self.zone.clone(),
            endpoint: self.endpoint.clone(),
            output_format: self.output_format.clone(),
            profiles: self.profiles.clone(),
        }
    }
}

/// Get the config directory path.
fn dirs_path() -> std::path::PathBuf {
    // Use $HOME/.cloudctl or current directory as fallback
    std::env::var("HOME")
        .map(|h| std::path::PathBuf::from(h).join(".cloudctl"))
        .unwrap_or_else(|_| std::path::PathBuf::from(".cloudctl"))
}

/// Load configuration from file.
pub fn load_config(path: Option<&Path>) -> Result<Config, CloudError> {
    let config_path = path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(Config::default_path);

    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| CloudError::Config {
                message: format!("Failed to read config file: {}", e),
                source: Some(Box::new(e)),
            })?;

        toml::from_str(&content)
            .map_err(|e| CloudError::Config {
                message: format!("Failed to parse config file: {}", e),
                source: Some(Box::new(e)),
            })
    } else {
        // Return default config if file doesn't exist
        Ok(Config::default())
    }
}

/// Save configuration to file.
pub fn save_config(config: &Config, path: Option<&Path>) -> Result<(), CloudError> {
    let config_path = path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(Config::default_path);

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| CloudError::Config {
                message: format!("Failed to create config directory: {}", e),
                source: Some(Box::new(e)),
            })?;
    }

    let content = toml::to_string_pretty(config)
        .map_err(|e| CloudError::Config {
            message: format!("Failed to serialize config: {}", e),
            source: Some(Box::new(e)),
        })?;

    std::fs::write(&config_path, content)
        .map_err(|e| CloudError::Config {
            message: format!("Failed to write config file: {}", e),
            source: Some(Box::new(e)),
        })
}
