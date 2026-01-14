#![allow(dead_code)] // Remove this once you start using the code

use std::{collections::HashMap, env, path::PathBuf, sync::Arc};

use arc_swap::{ArcSwap, Guard};
use color_eyre::Result;
use config::{Environment, File};
use directories::ProjectDirs;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

const DEFAULT_CONFIG: &str = include_str!("../.config/config.json5");

lazy_static! {
    static ref CONFIG: ArcSwap<Config> =
        ArcSwap::from_pointee(Config::load().expect("load initial config"));
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub data_dir: PathBuf,
    #[serde(default)]
    pub config_dir: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Default, Serialize)]
pub struct ProjectConfig {
    pub internal_name: String,
    pub jira_url: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default, flatten)]
    pub config: AppConfig,
    #[serde(default)]
    pub projects: HashMap<String, ProjectConfig>,
    pub default_project_key: String,
}

lazy_static! {
    pub static ref PROJECT_NAME: String = env!("CARGO_CRATE_NAME").to_uppercase().to_string();
    pub static ref DATA_FOLDER: Option<PathBuf> =
        env::var(format!("{}_DATA", PROJECT_NAME.clone()))
            .ok()
            .map(PathBuf::from);
    pub static ref CONFIG_FOLDER: Option<PathBuf> =
        env::var(format!("{}_CONFIG", PROJECT_NAME.clone()))
            .ok()
            .map(PathBuf::from);
}

impl Config {
    fn load() -> Result<Self, config::ConfigError> {
        let data_dir = get_data_dir();
        let config_dir = get_config_dir();

        let mut builder = config::Config::builder()
            .set_default("data_dir", data_dir.to_str().unwrap())?
            .set_default("config_dir", config_dir.to_str().unwrap())?
            .add_source(File::from_str(DEFAULT_CONFIG, config::FileFormat::Json5));

        let config_files = [
            ("config.json5", config::FileFormat::Json5),
            ("config.json", config::FileFormat::Json),
            ("config.yaml", config::FileFormat::Yaml),
            ("config.toml", config::FileFormat::Toml),
        ];
        for (file, format) in &config_files {
            let source = config::File::from(config_dir.join(file))
                .format(*format)
                .required(false);
            builder = builder.add_source(source);
        }

        let cfg: Self = builder
            .add_source(Environment::with_prefix("SHEET_SHARK"))
            .build()?
            .try_deserialize()?;

        Ok(cfg)
    }

    /// Obtain a long-term borrow of the config, e.g. for sharing across async barriers
    pub fn get_longterm() -> Arc<Self> {
        CONFIG.load_full()
    }

    /// Obtain a short-term borrow of the config
    pub fn get_quick() -> Guard<Arc<Self>> {
        CONFIG.load()
    }

    pub fn reload() -> Result<(), config::ConfigError> {
        let cfg = Self::load()?;
        CONFIG.store(Arc::new(cfg));
        Ok(())
    }

    #[cfg(test)]
    pub fn set_for_tests(config: Config) {
        CONFIG.store(Arc::new(config));
    }
}

pub fn get_data_dir() -> PathBuf {
    if let Some(s) = DATA_FOLDER.clone() {
        s
    } else if let Some(proj_dirs) = project_directory() {
        proj_dirs.data_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".data")
    }
}

pub fn get_config_dir() -> PathBuf {
    if let Some(s) = CONFIG_FOLDER.clone() {
        s
    } else if let Some(proj_dirs) = project_directory() {
        proj_dirs.config_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".config")
    }
}

fn project_directory() -> Option<ProjectDirs> {
    ProjectDirs::from("plus.lit", "", env!("CARGO_PKG_NAME"))
}
