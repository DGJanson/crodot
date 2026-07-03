// Code for managing a project

use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

// The basic project is a struct that holds
// some info on the crodot project

pub struct Project {
    pub name: Option<String>,
    pub directory: PathBuf,
    pub project_file: PathBuf,
    pub desc_file: Option<PathBuf>,
    pub file_short_list: Vec<PathBuf>,
    pub file_active_list: Vec<PathBuf>,
}

// We use a different that holds Strings instead of PathBufs
// We use this for writing and reading jsons
// Note this means we only can support UTF-8 valid paths!
#[derive(Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: Option<String>,
    pub directory: String,
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
        project_file: project_file_suggestion,
        desc_file: None,
        file_short_list: vec![],
        file_active_list: vec![],
    }
}

// A custom error we can use in case something goes wrong

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
    let json_string = match serde_json::to_string(&project_config) {
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
    let file_string = match std::fs::read_to_string(loc) {
        Ok(fs) => fs,
        Err(_) => return Err(ProjectError::NoFileError),
    };

    let project_config: ProjectConfig = serde_json::from_str(&file_string)
        .map_err(|err| ProjectError::FormatError(format!("{err}")))?;

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

    // Let us not fail completely when this happens
    let file_short_list: Result<Vec<String>, ProjectError> = project
        .file_short_list
        .iter()
        .map(|pb| pb_to_string(pb))
        .collect();

    let file_active_list: Result<Vec<String>, ProjectError> = project
        .file_active_list
        .iter()
        .map(|pb| pb_to_string(pb))
        .collect();

    Ok(ProjectConfig {
        name: project.name.clone(),
        directory,
        project_file,
        desc_file,
        file_short_list: file_short_list.unwrap_or_default(),
        file_active_list: file_active_list.unwrap_or_default(),
    })
}

/// Converts a project to a project from a read ProjectConfig.
/// In practice this means converting all Strings
/// To PathBufs
fn convert_to_project(project: ProjectConfig) -> Result<Project, ProjectError> {
    let directory = string_to_pb(&project.directory)?;
    let project_file = string_to_pb(&project.project_file)?;
    let desc_file = match project.desc_file.as_ref() {
        Some(df) => Some(string_to_pb(df)?),
        None => None,
    };

    // Let us not fail completely when this happens
    let file_short_list: Result<Vec<PathBuf>, ProjectError> = project
        .file_short_list
        .iter()
        .map(|pb| string_to_pb(pb))
        .collect();

    let file_active_list: Result<Vec<PathBuf>, ProjectError> = project
        .file_active_list
        .iter()
        .map(|pb| string_to_pb(pb))
        .collect();

    Ok(Project {
        name: project.name.clone(),
        directory,
        project_file,
        desc_file,
        file_short_list: file_short_list.unwrap_or_default(),
        file_active_list: file_active_list.unwrap_or_default(),
    })
}

fn pb_to_string(pb: &Path) -> Result<String, ProjectError> {
    let string_opt = pb.to_str();
    match string_opt {
        Some(string) => Ok(String::from(string)),
        None => Err(ProjectError::InvalidPathError),
    }
}

fn string_to_pb(string: &str) -> Result<PathBuf, ProjectError> {
    let pb_opt = PathBuf::from_str(string);
    match pb_opt {
        Ok(pb) => Ok(pb),
        Err(_) => Err(ProjectError::InvalidPathError),
    }
}
