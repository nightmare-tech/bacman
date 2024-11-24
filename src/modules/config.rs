use std::collections::HashMap;
use std::fs;
// Configuration management
// watch paths for backup
// backup path for offline backups
// host type ?
// credentials: cloud provider api key, cloud path
use toml;
use serde::{Deserialize};
use directories::ProjectDirs;
// pub fn get_project_dirs() {
//
// }
#[derive(Debug, Deserialize)]
struct Config {
    global: GlobalConfig,
    profiles: std::collections::HashMap<String, Profile>,
    backup_paths: Vec<BackupPath>,
}

#[derive(Debug, Deserialize)]
struct GlobalConfig {
    default_profile: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Profile {
    encrypt: bool,
    backup_method: Option<String>,
    backup_to: Option<String>,
    interval: Option<String>,
    enabled: bool,
}

#[derive(Debug, Deserialize)]
struct BackupPath {
    path: String,
    profile: Option<String>, // Optional because users might override settings directly
    encrypt: Option<bool>,
    backup_method: Option<String>,
    backup_to: Option<String>,
    interval: Option<String>,
    enabled: bool,
}

impl Config {
    pub fn resolve_backup_paths(&self) -> Vec<ResolvedBackupPath> {
        self.backup_paths
            .iter()
            .map(|path| {
                // Resolve the profile for the path, or use the global default profile
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
                ResolvedBackupPath {
                    path: path.path.clone(),
                    encrypt: path.encrypt.unwrap_or(false) // BackupPath encrypt takes priority
                        || profile.map_or(false, |p| p.encrypt),
                    backup_method: path
                        .backup_method
                        .clone()
                        .or_else(|| profile.and_then(|p| p.backup_method.clone())),
                    backup_to: path
                        .backup_to
                        .clone()
                        .or_else(|| profile.and_then(|p| p.backup_to.clone())),
                    interval: path
                        .interval
                        .clone()
                        .or_else(|| profile.and_then(|p| p.interval.clone())),
                    enabled: path.enabled && profile.map_or(true, |p| p.enabled),
                }
            })
            .collect()
    }
}



#[derive(Debug)]
pub struct ResolvedBackupPath {
    path: String,
    encrypt: bool,
    backup_method: Option<String>,
    backup_to: Option<String>,
    interval: Option<String>,
    enabled: bool,
}

pub fn deserialise_config() -> Vec<ResolvedBackupPath> {

    if let Some(proj_dirs) = ProjectDirs::from("", "", "bacman") {
        let config_dir = proj_dirs.config_dir();
        let config_file = fs::read_to_string(config_dir.join("config.toml"));
        let config: Config = match config_file {
            Ok(config) => toml::from_str(&config).unwrap(),
            Err(_) => panic!("Unable to read config file"),
        };

        // Resolve backup paths
        let resolved_paths = config.resolve_backup_paths();
        // Pint resolved paths
        // for path in &resolved_paths {
        //     dbg!("{:#?}", path);
        // }
        resolved_paths

    }
    else {
        panic!("Unable to read config file");
    }


}