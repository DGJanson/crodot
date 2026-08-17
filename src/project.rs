// Code for managing a project

use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

// The basic project is a struct that holds
// some info on the crodot project
pub struct PromptDir {
    pub description: String,
    pub path: PathBuf,
}

pub struct Project {
    pub name: Option<String>,
    /// Project ROOT directory
    pub directory: PathBuf,
    /// Sub dirs to be added to prompt
    pub prompt_dirs: Vec<PromptDir>,
    pub project_file: PathBuf,
    pub desc_file: Option<PathBuf>,
    pub file_short_list: Vec<PathBuf>,
    pub file_active_list: Vec<PathBuf>,
}

// We use a different that holds Strings instead of PathBufs
// We use this for writing and reading jsons
// Note this means we only can support UTF-8 valid paths!
#[derive(Serialize, Deserialize)]
pub struct PromptDirConfig {
    pub description: String,
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: Option<String>,
    pub directory: String,
    pub prompt_dirs: Vec<PromptDirConfig>,
    pub project_file: String,
    pub desc_file: Option<String>,
    pub file_short_list: Vec<String>,
    pub file_active_list: Vec<String>,
}

pub fn create_new_project(work_dir: PathBuf) -> Project {
    let project_file_suggestion = work_dir.join(".crodot.json");
    Project {
        name: None,
        directory: work_dir,
        prompt_dirs: vec![],
        project_file: project_file_suggestion,
        desc_file: None,
        file_short_list: vec![],
        file_active_list: vec![],
    }
}

// A custom error we can use in case something goes wrong

#[allow(clippy::enum_variant_names)]
#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("Could not write Project file")]
    WriteError,
    #[error("Project file not specified or not found")]
    NoFileError,
    #[error("Invalid format: {0}")]
    FormatError(String),
    #[error("Invalid path. We only support UTF-8 paths")]
    InvalidPathError,
}

// IO function for the project files

/// Tries to save the project file
/// Note that we need a project file location :)
/// Make sure it is set in the Project struct
pub fn save_project_file(project: &Project) -> Result<(), ProjectError> {
    // First check if file is defined in project
    let project_path = &project.project_file;
    let project_config = convert_to_project_config(project)?;
    let json_string = match serde_json::to_string_pretty(&project_config) {
        Ok(js) => js,
        Err(err) => return Err(ProjectError::FormatError(format!("{err}"))),
    };

    let write_result = std::fs::write(project_path, json_string);
    match write_result {
        Ok(()) => Ok(()),
        Err(_) => Err(ProjectError::WriteError),
    }
}

pub fn read_project_file_with_location_string(loc: &str) -> Result<Project, ProjectError> {
    let project_path = PathBuf::from(loc);
    read_project_file(project_path)
}

pub fn read_project_file(loc: PathBuf) -> Result<Project, ProjectError> {
    // read file to string
    let file_string = match std::fs::read_to_string(loc.clone()) {
        Ok(fs) => fs,
        Err(_) => return Err(ProjectError::NoFileError),
    };

    let project_config: ProjectConfig = serde_json::from_str(&file_string)
        .map_err(|err| ProjectError::FormatError(format!("{err}")))?;

    // convert
    convert_to_project(project_config)
}

/// Converts a project to a projectConfig.
/// In practice this means converting all PathBufs
/// To String
fn convert_to_project_config(project: &Project) -> Result<ProjectConfig, ProjectError> {
    let directory = pb_to_string(&project.directory)?;
    let project_file = pb_to_string(&project.project_file)?;
    let desc_file = match project.desc_file.as_ref() {
        Some(df) => Some(pb_to_string(df)?),
        None => None,
    };

    let file_short_list: Vec<String> = project
        .file_short_list
        .iter()
        .filter_map(|pb| pb_to_string(pb).ok())
        .collect();

    let file_active_list: Vec<String> = project
        .file_active_list
        .iter()
        .filter_map(|pb| pb_to_string(pb).ok())
        .collect();

    let prompt_dirs: Vec<PromptDirConfig> = project
        .prompt_dirs
        .iter()
        .filter_map(|pd| match pb_to_string(&pd.path) {
            Ok(path_str) => Some(PromptDirConfig {
                description: pd.description.clone(),
                path: path_str,
            }),
            Err(_) => None,
        })
        .collect();

    Ok(ProjectConfig {
        name: project.name.clone(),
        directory,
        prompt_dirs,
        project_file,
        desc_file,
        file_short_list,
        file_active_list,
    })
}

/// Converts a ProjectConfig to a Project.
/// In practice this means converting all Strings
/// To PathBufs
fn convert_to_project(project_config: ProjectConfig) -> Result<Project, ProjectError> {
    let directory = string_to_pb(&project_config.directory)?;
    let project_file = string_to_pb(&project_config.project_file)?;
    let desc_file = match project_config.desc_file.as_ref() {
        Some(df) => Some(string_to_pb(df)?),
        None => None,
    };

    let file_short_list: Vec<PathBuf> = project_config
        .file_short_list
        .iter()
        .filter_map(|s| string_to_pb(s).ok())
        .collect();

    let file_active_list: Vec<PathBuf> = project_config
        .file_active_list
        .iter()
        .filter_map(|s| string_to_pb(s).ok())
        .collect();

    let prompt_dirs: Vec<PromptDir> = project_config
        .prompt_dirs
        .iter()
        .filter_map(|pdc| match string_to_pb(&pdc.path) {
            Ok(pb) => Some(PromptDir {
                description: pdc.description.clone(),
                path: pb,
            }),
            Err(_) => None,
        })
        .collect();

    Ok(Project {
        name: project_config.name.clone(),
        directory,
        prompt_dirs,
        project_file,
        desc_file,
        file_short_list,
        file_active_list,
    })
}

fn pb_to_string(pb: &Path) -> Result<String, ProjectError> {
    match pb.to_str() {
        Some(string) => Ok(string.to_string()),
        None => Err(ProjectError::InvalidPathError),
    }
}

fn string_to_pb(string: &str) -> Result<PathBuf, ProjectError> {
    PathBuf::from_str(string).map_err(|_| ProjectError::InvalidPathError)
}
