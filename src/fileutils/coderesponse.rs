// Script for handling coding response from LLM

use std::path::{Path, PathBuf};

use crate::AppState;

#[derive(Debug)]
pub struct CodingResponse {
    pub error: Option<String>,
    pub message: String,
    pub code: Vec<FileChange>,
}

#[derive(Debug)]
pub struct FileChange {
    pub filename: String,
    pub mode: ChangeMode,
    pub description: String,
    pub file_contents: String,
}

#[derive(Debug, PartialEq)]
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

// We make a custom parsing error for the response
// We only need it in this module
#[derive(Debug, thiserror::Error)]
enum CodingResponseError {
    #[error("Invalid line: {0}")]
    InvalidLine(String),
    #[error("Unexpected format: {0}")]
    InvalidFormat(String),
}

/// Parse an LLM coding response
/// Note we use a somewhat custom text format
/// (see the crodot system prompt in this repo)
/// Will use the error of the AppState
/// in case of any error
pub fn parse_coding_response(state: &mut AppState, response: String) {
    let clean_response = sanitize_llm_response(&response);
    match coding_response_handler(clean_response) {
        Ok(cr) => state.llm_response_code_response = Some(cr),
        Err(e) => {
            state.llm_response_code_response = None;
            state.error_msg = Some(format!("{e}"));
        }
    }
}

fn coding_response_handler(response: &str) -> Result<CodingResponse, CodingResponseError> {
    // We can parse the response linearly
    // some housekeeping vars
    let mut file_changes: Vec<FileChange> = Vec::new();
    let mut description = String::new();
    let mut error: Option<String> = None;

    let mut cur_filename = String::new();
    let mut cur_filemode = ChangeMode::New;
    let mut cur_file_desc = String::new();
    let mut file_contents: Vec<&str> = Vec::new(); // this will have the code

    let mut finished_header = false;
    let mut read_first_line = false;
    let mut read_description = false;
    let mut read_error = false;

    let mut file_header_counter = 0; // to keep track of file vars
    let mut collect_code = false;
    // iterate
    for line in response.lines() {
        if !read_first_line {
            if line.trim() != "- Response" {
                return Err(CodingResponseError::InvalidLine(format!(
                    "First line is not what it should be: {line}"
                )));
            } else {
                read_first_line = true;
                continue;
            }
        }
        if !finished_header {
            if line.trim() == "- Error" {
                read_error = true;
                continue;
            } else if line.trim() == "- Message" {
                read_description = true;
                continue;
            } else if read_error {
                // Set error if line not empty
                if !line.trim().is_empty() {
                    error = Some(String::from(line));
                }
                read_error = false;
            } else if read_description {
                description = String::from(line);
                read_description = false;
            } else if line.trim() == "- Files" {
                finished_header = true;
                continue;
            } else if line.trim().is_empty() {
                // bit of an edge case. But ignore empty lines
                continue;
            } else {
                return Err(CodingResponseError::InvalidLine(format!(
                    "Unexpected header line in response: {line}",
                )));
            }
        }
        if finished_header {
            // Now we just read files
            if file_header_counter <= 2 {
                // Skip completely empty lines
                if line.trim().is_empty() {
                    continue;
                }
                // in file header
                if !line.starts_with("-!-") || line.len() <= 4 {
                    return Err(CodingResponseError::InvalidLine(format!(
                        "Unexpected file header line in response: {line}"
                    )));
                }
                if file_header_counter == 0 {
                    let Some(filename) = line.strip_prefix("-!- ") else {
                        return Err(CodingResponseError::InvalidLine(format!(
                            "Unexpected file header line in response: {line}"
                        )));
                    };
                    cur_filename = filename.to_string();
                    file_header_counter += 1;
                } else if file_header_counter == 1 {
                    let Some(mode_str) = line.strip_prefix("-!- ") else {
                        return Err(CodingResponseError::InvalidLine(format!(
                            "Unexpected file header line in response: {line}"
                        )));
                    };
                    match mode_str.trim().to_lowercase().as_str() {
                        "new" => cur_filemode = ChangeMode::New,
                        "edit" => cur_filemode = ChangeMode::Edit,
                        _ => {
                            return Err(CodingResponseError::InvalidLine(format!(
                                "Invalid file mode: {line}"
                            )));
                        }
                    }
                    file_header_counter += 1;
                } else if file_header_counter == 2 {
                    let Some(file_desc) = line.strip_prefix("-!- ") else {
                        return Err(CodingResponseError::InvalidLine(format!(
                            "Unexpected file header line in response: {line}"
                        )));
                    };
                    cur_file_desc = file_desc.to_string();
                    file_header_counter += 1;
                    collect_code = true; // Finished header
                }
            } else {
                // Out of file header
                if line.trim() == "-!-" {
                    // End of code marker
                    // Wrap up the code collection and create a FileChange object
                    let fc = FileChange {
                        filename: cur_filename,
                        mode: cur_filemode,
                        description: cur_file_desc,
                        file_contents: file_contents.join("\n"),
                    };
                    file_changes.push(fc);
                    // reset the vars
                    cur_filename = String::new();
                    cur_filemode = ChangeMode::New;
                    cur_file_desc = String::new();
                    file_contents.clear();
                    // reset state
                    collect_code = false;
                    file_header_counter = 0;
                } else if collect_code {
                    // We are in file contents now
                    file_contents.push(line);
                } else if line.trim().is_empty() {
                    // This is an edge case for trailing empty lines
                    // simply ignore
                    continue;
                } else {
                    // this is invalid state, this should not be possible
                    return Err(CodingResponseError::InvalidFormat(format!(
                        "Unexpected format / ordering of header markers. Line: {line}",
                    )));
                }
            }
        }
    }
    // We have seen all lines
    // Create the coding response and set it
    let cr = CodingResponse {
        message: description,
        error,
        code: file_changes,
    };
    Ok(cr)
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

pub fn normalize_filepath(source_dir: &Path, filename: &str) -> PathBuf {
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

    use super::coding_response_handler;

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

    #[test]
    fn read_code_response() {
        let cr_string = r#"- Response
- Error

- Message
I have edited the main file, it now call the add function new file add.py.
- Files
-!- src/module/main.py
-!- edit
-!- Added add function call to main function
"""
Main entry point of the program  
"""
from .add import add

def main():
    print("Hello World")
    # call add function here
    add(2, 3)
-!-
-!- src/module/add.py
-!- new
-!- Create add function
"""
Module with some add functions
"""

def add(x, y):
    return x + y
-!-

"#;
        let cr_res = coding_response_handler(cr_string);
        assert!(cr_res.is_ok());
        match cr_res {
            Ok(cr) => {
                assert!(cr.error.is_none());
                assert!(cr.code.len() == 2);
            }
            Err(e) => {
                print!("{e}");
                panic!();
            }
        }
    }
}
