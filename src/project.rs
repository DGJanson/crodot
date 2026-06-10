// Code for managing a project

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// The basic project is a struct that holds
// some info on the crodot project

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(with = "path_handling")]
    pub directory: PathBuf,
    #[serde(with = "path_handling")]
    pub project_file: PathBuf,
}

pub fn create_new_project(work_dir: PathBuf) -> Project {
    let project_file_suggestion = work_dir.join(".crodot.json");
    Project {
        name: None,
        description: None,
        directory: work_dir,
        project_file: project_file_suggestion,
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
}

// IO function for the project files

/// Tries to save the project file
/// Note that we need a project file location :)
/// Make sure it is set in the Project struct
pub fn save_project_file(project: &Project) -> Result<(), ProjectError> {
    // First check if file is defined in project
    let project_path = &project.project_file;
    let json_string = match serde_json::to_string(&project) {
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

    let project_opt = serde_json::from_str(&file_string);
    match project_opt {
        Ok(project) => Ok(project),
        Err(err) => Err(ProjectError::FormatError(format!("{err}"))),
    }
}

// We create a small module that helps serde with
// the pathbufs in the project.
// Note we accept that this way ONLY supports UTF-8 paths
#[allow(clippy::ptr_arg)]
mod path_handling {
    use std::ffi::OsString;

    use super::*;
    use serde::de::Deserializer;
    use serde::ser::Serializer;

    pub fn serialize<S>(p: &PathBuf, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        p.as_os_str().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(OsString::deserialize(deserializer)?.into())
    }
}
