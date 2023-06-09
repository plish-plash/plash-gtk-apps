use std::{path::Path, sync::Mutex};

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

static STATUSES: Mutex<Option<Vec<String>>> = Mutex::new(None);
static PROJECT_TYPES: Mutex<Option<Vec<ProjectType>>> = Mutex::new(None);

pub fn status_to_index(status: &str) -> usize {
    if let Some(statuses) = STATUSES.try_lock().unwrap().as_ref() {
        statuses.iter().position(|s| s == status).unwrap_or(0)
    } else {
        0
    }
}

pub fn project_type_to_index(project_type: &str) -> usize {
    if let Some(project_types) = PROJECT_TYPES.try_lock().unwrap().as_ref() {
        project_types
            .iter()
            .position(|t| t.name == project_type)
            .unwrap_or(0)
    } else {
        0
    }
}

pub fn project_type_application(project_type: &str) -> Option<DesktopAppInfo> {
    if let Some(project_types) = PROJECT_TYPES.try_lock().unwrap().as_ref() {
        if let Some(project_type) = project_types.iter().find(|t| t.name == project_type) {
            return DesktopAppInfo::new(&project_type.application);
        }
    }
    None
}

fn deserialize<T: DeserializeOwned>(path: &Path) -> Result<T, String> {
    let map_err_io = |error| format!("Error reading {}: {}", path.display(), error);
    let map_err_toml = |error| format!("Error reading {}: {}", path.display(), error);
    toml::from_str(&std::fs::read_to_string(path).map_err(map_err_io)?).map_err(map_err_toml)
}

pub fn load_config(config_dir: &Path) -> Result<(), String> {
    let statuses: StatusesConfig = deserialize(&config_dir.join("status.toml"))?;
    STATUSES.try_lock().unwrap().replace(statuses.status);
    let project_types: TypesConfig = deserialize(&config_dir.join("type.toml"))?;
    PROJECT_TYPES
        .try_lock()
        .unwrap()
        .replace(project_types.r#type);
    Ok(())
}

pub fn load_projects(projects_file: &Path) -> Result<Vec<ProjectInfo>, String> {
    let projects: ProjectsConfig = deserialize(projects_file)?;
    Ok(projects.project)
}
