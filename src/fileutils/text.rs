// Some functions for reading text files

use std::{fs::read_to_string, path::PathBuf, str::FromStr};

use super::errors::FileError;

pub fn read_file_text_from_loc(fileloc: &str) -> Result<String, FileError> {
    let pb = PathBuf::from_str(fileloc)
        .map_err(|_| FileError::FileNotFoundError(String::from(fileloc)))?;
    read_file_text(&pb)
}

pub fn read_file_text(pb: &PathBuf) -> Result<String, FileError> {
    let contents_opt = read_to_string(pb);
    match contents_opt {
        Ok(s) => Ok(s),
        Err(_) => Err(FileError::FileNotReadError(
            pb.to_string_lossy().to_string(),
        )),
    }
}
