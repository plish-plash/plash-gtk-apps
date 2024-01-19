use std::path::Path;

use gtk::gio::DesktopAppInfo;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub project_type: String,
    pub status: String,
    pub last_opened: i64,
    pub path: String,
    pub notes: String,
}

#[derive(Deserialize)]
struct ProjectType {
    name: String,
    application: String,
}

#[derive(Deserialize)]
struct StatusesConfig {
    status: Vec<String>,
}

#[derive(Deserialize)]
struct TypesConfig {
    r#type: Vec<ProjectType>,
}

#[derive(Serialize, Deserialize)]
struct ProjectsConfig {
    project: Vec<ProjectInfo>,
}

#[derive(Default)]
pub struct AppConfig {
    statuses: Vec<String>,
    project_types: Vec<ProjectType>,
}

impl AppConfig {
    pub fn statuses(&self) -> &[String] {
        &self.statuses
    }
    pub fn status_index(&self, status: &str) -> usize {
        self.statuses.iter().position(|s| s == status).unwrap_or(0)
    }
    pub fn project_type_index(&self, project_type: &str) -> usize {
        self.project_types
            .iter()
            .position(|t| t.name == project_type)
            .unwrap_or(0)
    }
    pub fn project_type_application(&self, project_type: &str) -> Option<DesktopAppInfo> {
        self.project_types
            .iter()
            .find(|t| t.name == project_type)
            .and_then(|t| DesktopAppInfo::new(&t.application))
    }
}

fn deserialize<T: DeserializeOwned>(path: &Path) -> Result<T, String> {
    let map_err_io = |error| format!("Error reading {}: {}", path.display(), error);
    let map_err_toml = |error| format!("Error reading {}: {}", path.display(), error);
    toml::from_str(&std::fs::read_to_string(path).map_err(map_err_io)?).map_err(map_err_toml)
}

pub fn load_config(config_dir: &Path) -> Result<AppConfig, String> {
    let statuses: StatusesConfig = deserialize(&config_dir.join("status.toml"))?;
    let project_types: TypesConfig = deserialize(&config_dir.join("type.toml"))?;
    Ok(AppConfig {
        statuses: statuses.status,
        project_types: project_types.r#type,
    })
}

pub fn load_projects(projects_file: &Path) -> Result<Vec<ProjectInfo>, String> {
    let projects: ProjectsConfig = deserialize(projects_file)?;
    Ok(projects.project)
}
