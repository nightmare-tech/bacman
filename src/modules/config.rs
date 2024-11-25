use std::collections::HashMap;
use std::fs;
use toml;
use serde::Deserialize;
use directories::ProjectDirs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub global: GlobalConfig,
    pub profiles: HashMap<String, Profile>,
    pub backup_paths: Vec<BackupPath>,
}

#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    pub default_profile: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Profile {
    pub encrypt: Option<bool>,
    pub backup_method: Option<Vec<String>>,
    pub backup_to: Option<String>,
    pub interval: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BackupPath {
    pub path: String,
    pub profile: Option<String>,
    // All fields optional to allow complete override flexibility
    pub encrypt: Option<bool>,
    pub backup_method: Option<Vec<String>>,
    pub backup_to: Option<String>,
    pub interval: Option<String>,
}

#[derive(Debug)]
pub struct ResolvedBackupPath {
    pub path: String,
    pub encrypt: Option<bool>,
    pub backup_method: Option<Vec<String>>,
    pub backup_to: Option<String>,
    pub interval: Option<String>,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Invalid configuration: {0}")]
    Invalid(String),
}

impl Config {
    pub fn resolve_backup_paths(&self) -> Vec<ResolvedBackupPath> {
        self.backup_paths
            .iter()
            .map(|path| {
                // Get the profile if specified, or use default profile
                let profile = path
                    .profile
                    .as_ref()
                    .and_then(|p| self.profiles.get(p))
                    .or_else(|| {
                        self.global
                            .default_profile
                            .as_ref()
                            .and_then(|default| self.profiles.get(default))
                    });

                // Resolve each field, prioritizing path-specific settings over profile settings
                ResolvedBackupPath {
                    path: path.path.clone(),
                    encrypt: path.encrypt.or_else(|| profile.and_then(|p| p.encrypt)),
                    backup_method: path.backup_method.clone().or_else(|| {
                        profile.and_then(|p| p.backup_method.clone())
                    }),
                    backup_to: path.backup_to.clone().or_else(|| {
                        profile.and_then(|p| p.backup_to.clone())
                    }),
                    interval: path.interval.clone().or_else(|| {
                        profile.and_then(|p| p.interval.clone())
                    }),
                }
            })
            .collect()
    }
    pub fn validate(&self) -> Result<(), ConfigError> {
        let mut errors = Vec::new();

        // Validate default profile exists if specified
        if let Some(default) = &self.global.default_profile {
            if !self.profiles.contains_key(default) {
                errors.push(format!("Default profile '{}' not found", default));
            }
        }

        // Validate backup paths and their profiles
        for path in &self.backup_paths {
            // Check path exists
            if !Path::new(&path.path).exists() {
                errors.push(format!("Path does not exist: {}", path.path));
            }

            // Validate profile reference
            if let Some(profile_name) = &path.profile {
                if !self.profiles.contains_key(profile_name) {
                    errors.push(format!("Referenced profile not found: {}", profile_name));
                }
            }

            // Validate backup method
            if let Some(methods) = &path.backup_method {
                for method in methods {
                    match method.to_lowercase().as_str() {
                        "local" | "git" | "gdrive" | "pdrive" | "dropbox" => (),
                        _ => errors.push(format!("Invalid backup method: {}", method)),
                    }
                }
            }

            // Validate interval format if specified
            if let Some(interval) = &path.interval {
                if !interval.chars().any(|c| c.is_digit(10))
                    || !interval.ends_with(|c| matches!(c, 'd' | 'h' | 'm')) {
                    errors.push(format!("Invalid interval format: {}", interval));
                }
            }

            // Validate backup destination if specified
            if let Some(dest) = &path.backup_to {
                if dest.starts_with('/') || dest.starts_with("./") {
                    if !Path::new(dest).exists() {
                        errors.push(format!("Backup destination not accessible: {}", dest));
                    }
                } else if !dest.starts_with("git@") && !dest.starts_with("https://") {
                    errors.push(format!("Invalid backup destination format: {}", dest));
                }
            }
        }

        // Validate resolved paths have required fields
        let resolved = self.resolve_backup_paths();
        for path in resolved {
            if path.backup_method.is_none() {
                errors.push(format!("No backup method specified for path: {}", path.path));
            }
            if path.backup_to.is_none() {
                errors.push(format!("No backup destination specified for path: {}", path.path));
            }
        }



        if errors.is_empty() {
            Ok(())
        } else {
            Err(ConfigError::Invalid(errors.join("\n")))
        }
    }

}

pub fn deserialize_config() -> Result<Vec<ResolvedBackupPath>, String> {
    ProjectDirs::from("", "", "bacman")
        .ok_or_else(|| "Could not determine project directories".to_string())
        .and_then(|proj_dirs| {
            let config_path = proj_dirs.config_dir().join("config.toml");
            fs::read_to_string(&config_path)
                .map_err(|e| format!("Failed to read config file: {}", e))
                .and_then(|config_str| {
                    let config = toml::from_str::<Config>(&config_str)
                        .map_err(|e| format!("Failed to parse config: {}", e))?;

                    config.validate()
                        .map_err(|e| e.to_string())?;

                    Ok(config.resolve_backup_paths())
                })
        })
}

pub fn extract_paths(configs: &Vec<ResolvedBackupPath>) -> Vec<String> {
    let mut paths: Vec<String> = vec![];
    for config in configs {
        let i = config.path.clone();
        paths.push(i);
    }
    paths
}
