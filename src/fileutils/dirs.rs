use std::fs;
use std::path::PathBuf;

/// Creates a String representation of the
/// dir structure of the folder passed.
/// If no valid dir or nothing found, will
/// return empty string
pub fn create_dir_structure(dir: &PathBuf) -> String {
    if !dir.is_dir() {
        return String::from("");
    }

    // We use format, even though it may be inefficient
    let mut result = String::from("```\n");

    result = handle_single_dir(&dir, 0, result);

    format!("{}\n```", result)
}

fn handle_single_dir(dir: &PathBuf, level: usize, mut result: String) -> String {
    result = format!(
        "{}{}- {}\n",
        result,
        "| ".repeat(level),
        dir.file_name().unwrap_or_default().to_string_lossy()
    );

    // Two loops, first files, then dirs
    if let Ok(ls) = fs::read_dir(dir) {
        for entry in ls {
            if let Ok(file) = entry {
                let file_path = file.path();
                if file_path.is_file() {
                    // simpy print the file
                    result = format!(
                        "{}{}| {}\n",
                        result,
                        "| ".repeat(level),
                        file_path.file_name().unwrap_or_default().to_string_lossy()
                    );
                }
            }
        }
    }
    if let Ok(ls) = fs::read_dir(dir) {
        for entry in ls {
            if let Ok(file) = entry {
                let file_path = file.path();
                if file_path.is_dir() {
                    // Now we recursively call this function
                    result = handle_single_dir(&file_path, level + 1, result);
                }
            }
        }
    }

    result
}
