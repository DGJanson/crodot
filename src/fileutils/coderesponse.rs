// Script for handling coding response from LLM

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct CodingResponse {
    pub error: Option<String>,
    pub message: String,
    pub code: Vec<FileChange>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileChange {
    pub filename: String,
    pub mode: ChangeMode,
    pub description: String,
    pub file_contents: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ChangeMode {
    New,
    Edit,
}

impl std::fmt::Display for ChangeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New => write!(f, "New file"),
            Self::Edit => write!(f, "Existing file"),
        }
    }
}

pub fn parse_coding_response(state: &mut AppState, response: String) {
    let clean_response = sanitize_llm_response(&response);
    let code_response_opt: Result<CodingResponse, serde_json::Error> =
        serde_json::from_str(clean_response);
    match code_response_opt {
        Ok(code_response) => {
            state.llm_response_code_response = Some(code_response);
        }
        Err(e) => {
            state.error_msg = Some(format!(
                "Coud not parse coding response. Try again. Error message: {}, \n\n {}",
                e, response
            ));
            state.llm_response_state = crate::LlmResponseState::Idle;
        }
    }
}

/// Cleans the LLM response of markdown
/// It often starts with ```json and ends with ```
/// Which will trip the parser, so we remove them
fn sanitize_llm_response(input: &str) -> &str {
    let mut cleaned = input.trim();
    if cleaned.starts_with("```") {
        if let Some(newline_pos) = cleaned.find("\n") {
            cleaned = &cleaned[newline_pos + 1..]
        }
    }

    if cleaned.ends_with("```") {
        cleaned = &cleaned[..cleaned.len() - 3];
    }

    cleaned
}

/// Function to write the file contents of a coding response
/// Based on the state vars
/// Return true if write successful
pub fn write_code_response_file(state: &mut AppState, override_filepath: &str) -> bool {
    // get rid of edge cases
    if state.current_project.is_none() {
        return false;
    }
    if state.llm_response_code_response.is_none() {
        return false;
    }

    // First get the required info
    // We unwrap a lot, but check everything beforehand
    //   crashes because of wraps should not be possible
    let project = state.current_project.as_ref().unwrap();
    let code_response = state.llm_response_code_response.as_ref().unwrap();
    let selected_index = state.llm_response_code_selected;
    if selected_index >= code_response.code.len() {
        state.error_msg = Some(String::from("Invalid code response selected"));
        return false;
    }
    let file_change = code_response.code.get(selected_index).unwrap();

    // We must check several things before we can surely write the file
    let filepath = if override_filepath.is_empty() {
        normalize_filepath(&project.directory, &file_change.filename)
    } else {
        PathBuf::from(override_filepath)
    };

    let write_result = std::fs::write(&filepath, &file_change.file_contents);

    match write_result {
        Ok(_) => true,
        Err(e) => {
            state.error_msg = Some(format!(
                "Error whilst writing file: {}. Message: {:?}",
                &filepath.display(),
                e
            ));
            false
        }
    }
}

pub fn normalize_filepath(source_dir: &PathBuf, filename: &str) -> PathBuf {
    // check if filename starts with seperator
    let rel_filename = filename.trim_start_matches("/").trim_start_matches("\\");

    // check if filename starts with the source dir
    // We accept the default, it simply means we can not detect
    //  if the filename starts with the sourcedir...
    let filepath = Path::new(rel_filename);

    // for now we simply check one dir
    let final_source_dir = source_dir.file_name().unwrap_or_default();

    // Strip dir, or simply keep the original
    let trimmed_filepath = filepath.strip_prefix(final_source_dir).unwrap_or(filepath);

    // Simply return the join
    source_dir.join(trimmed_filepath)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    // Contains some test for file comparisons
    use super::normalize_filepath;

    #[test]
    fn test_normalize() {
        assert_eq!(
            normalize_filepath(&PathBuf::from("/test/src/"), "test.rs"),
            PathBuf::from("/test/src/test.rs")
        );
        assert_eq!(
            normalize_filepath(&PathBuf::from("/test/src/"), "/test.rs"),
            PathBuf::from("/test/src/test.rs")
        );
        assert_eq!(
            normalize_filepath(&PathBuf::from("/test/src/"), "/src/test.rs"),
            PathBuf::from("/test/src/test.rs")
        );
        assert_eq!(
            normalize_filepath(&PathBuf::from("/test/"), "/src/test.rs"),
            PathBuf::from("/test/src/test.rs")
        );
        assert_eq!(
            normalize_filepath(&PathBuf::from("/test/src/src"), "/src/test.rs"),
            PathBuf::from("/test/src/src/test.rs")
        );
        assert_eq!(
            normalize_filepath(&PathBuf::from("/test/src/src"), "\\src/test.rs"),
            PathBuf::from("/test/src/src/test.rs")
        );
    }
}
