#![allow(dead_code)] // Remove this once you start using the code

use std::{collections::HashMap, env, path::PathBuf, sync::Arc};

use arc_swap::{ArcSwap, Guard};
use color_eyre::Result;
use config::{Environment, File};
use directories::ProjectDirs;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use time::{Date, format_description::FormatItem, macros::format_description};

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
    #[serde(default, deserialize_with = "deserialize_valid_from")]
    pub valid_from: Option<Date>,
}

const ISO_LOCAL_DATE: &[FormatItem<'static>] = format_description!("[year]-[month]-[day]");

fn deserialize_valid_from<'de, D>(deserializer: D) -> Result<Option<Date>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    value
        .map(|value| Date::parse(&value, ISO_LOCAL_DATE).map_err(serde::de::Error::custom))
        .transpose()
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ProjectDefinitions {
    Single(ProjectConfig),
    Versions(Vec<ProjectConfig>),
}

fn deserialize_projects<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, Vec<ProjectConfig>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let definitions = HashMap::<String, ProjectDefinitions>::deserialize(deserializer)?;
    Ok(definitions
        .into_iter()
        .map(|(key, definition)| {
            let versions = match definition {
                ProjectDefinitions::Single(project) => vec![project],
                ProjectDefinitions::Versions(projects) => projects,
            };
            (key, versions)
        })
        .collect())
}

#[derive(Clone, Debug, Deserialize, Default, Serialize)]
pub struct AzureConfig {
    #[serde(default = "default_container_name")]
    pub container_name: Option<String>,
    // Containers -> shark-exports -> Settings -> SAS -> account key, Create+Write
    pub blob_sas_url: Option<String>,
    #[serde(default = "default_debounce_seconds")]
    pub debounce_seconds: u64,
}

fn default_container_name() -> Option<String> {
    Some("shark-exports".to_string())
}

fn default_debounce_seconds() -> u64 {
    10
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default, flatten)]
    pub config: AppConfig,
    #[serde(default, deserialize_with = "deserialize_projects")]
    pub projects: HashMap<String, Vec<ProjectConfig>>,
    pub default_project_key: String,
    #[serde(default)]
    pub azure: Option<AzureConfig>,
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

        cfg.validate_projects()
            .map_err(config::ConfigError::Message)?;

        Ok(cfg)
    }

    /// Resolve a project configuration for a particular calendar day.
    pub fn project_for_day(&self, key: &str, day: Date) -> Option<ProjectConfig> {
        let versions = self.projects.get(key)?;
        versions
            .iter()
            .take_while(|project| {
                project
                    .valid_from
                    .is_none_or(|valid_from| valid_from <= day)
            })
            .last()
            .cloned()
    }

    fn validate_projects(&self) -> Result<(), String> {
        for (key, versions) in &self.projects {
            if versions.is_empty() {
                return Err(format!("project {key:?} must have at least one definition"));
            }
            if versions.len() == 1 && versions[0].valid_from.is_some() {
                return Err(format!(
                    "project {key:?}: valid_from is only supported when defining project versions as a list"
                ));
            }
            let mut previous = None;
            for (index, project) in versions.iter().enumerate() {
                let Some(valid_from) = project.valid_from else {
                    if index == 0 {
                        continue;
                    }
                    return Err(format!(
                        "project {key:?} version {} must specify valid_from",
                        index + 1
                    ));
                };
                if previous.is_some_and(|previous| valid_from <= previous) {
                    return Err(format!(
                        "project {key:?} version {} valid_from dates must be strictly increasing",
                        index + 1
                    ));
                }
                previous = Some(valid_from);
            }
        }
        Ok(())
    }

    #[cfg(test)]
    fn from_yaml(yaml: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config: Self = config::Config::builder()
            .add_source(File::from_str(yaml, config::FileFormat::Yaml))
            .build()?
            .try_deserialize()?;
        config.validate_projects().map_err(|error| {
            Box::<dyn std::error::Error>::from(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                error,
            ))
        })?;
        Ok(config)
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

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    #[test]
    fn supports_existing_single_project_format() {
        let config = Config::from_yaml(
            "default_project_key: E\nprojects:\n  E:\n    internal_name: Example\n",
        )
        .unwrap();

        let project = config.project_for_day("E", date!(1900 - 01 - 01)).unwrap();
        assert_eq!(project.internal_name, "Example");
        assert!(
            config
                .project_for_day("missing", date!(2026 - 01 - 01))
                .is_none()
        );
    }

    #[test]
    fn resolves_project_versions_at_inclusive_start_dates() {
        let config = Config::from_yaml(
            "default_project_key: E\nprojects:\n  E:\n    - internal_name: Earlier\n    - internal_name: Current\n      valid_from: 2026-01-01\n",
        )
        .unwrap();

        assert_eq!(
            config
                .project_for_day("E", date!(2025 - 12 - 31))
                .unwrap()
                .internal_name,
            "Earlier"
        );
        assert_eq!(
            config
                .project_for_day("E", date!(2026 - 01 - 01))
                .unwrap()
                .internal_name,
            "Current"
        );
        assert_eq!(
            config
                .project_for_day("E", date!(2100 - 01 - 01))
                .unwrap()
                .internal_name,
            "Current"
        );
    }

    #[test]
    fn first_version_can_start_in_the_future() {
        let config = Config::from_yaml(
            "default_project_key: E\nprojects:\n  E:\n    - internal_name: Future\n      valid_from: 2026-01-01\n    - internal_name: Later\n      valid_from: 2027-01-01\n",
        )
        .unwrap();

        assert!(config.project_for_day("E", date!(2025 - 12 - 31)).is_none());
        assert_eq!(
            config
                .project_for_day("E", date!(2026 - 01 - 01))
                .unwrap()
                .internal_name,
            "Future"
        );
    }

    #[test]
    fn rejects_invalid_version_orders_and_missing_dates() {
        for yaml in [
            "default_project_key: E\nprojects:\n  E:\n    - internal_name: First\n      valid_from: 2026-02-01\n    - internal_name: Second\n      valid_from: 2026-01-01\n",
            "default_project_key: E\nprojects:\n  E:\n    - internal_name: First\n    - internal_name: Second\n      valid_from: 2026-01-01\n    - internal_name: Third\n",
            "default_project_key: E\nprojects:\n  E:\n    - internal_name: Single\n      valid_from: 2026-01-01\n",
        ] {
            assert!(Config::from_yaml(yaml).is_err());
        }
    }

    #[test]
    fn valid_from_requires_an_iso_local_date() {
        let config = Config::from_yaml(
            "default_project_key: E\nprojects:\n  E:\n    - internal_name: First\n    - internal_name: Second\n      valid_from: 2026-01-01T00:00:00Z\n",
        );

        assert!(config.is_err());
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
