use std::collections::HashSet;
use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};

use regex::Regex;
use serde_json::Value;

fn main() -> Result<(), io::Error> {
    let en_path = "langs/en-US.json";
    let ko_path = "langs/ko-KR.json";
    let _en_key_list = extract_keys_from_json(en_path)?;
    let _ko_key_list = extract_keys_from_json(ko_path)?;
    let rs_file_paths = get_rs_files("src")?;

    let re = Regex::new(r#""([^"\\]*(\\.[^"\\]*)*)""#)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let _all_strings: HashSet<String> = rs_file_paths
        .into_iter()
        .map(|path| collect_strings_from_file(&path, &re))
        .flat_map(|result| result.into_iter().flatten())
        .collect::<HashSet<_>>();

    Ok(())
}

fn extract_keys_from_json(path: &str) -> Result<HashSet<String>, io::Error> {
    let content = fs::read_to_string(path)
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("File error: {e}")))?;

    let json: Value = serde_json::from_str(&content)
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("JSON error: {e}")))?;

    if let Value::Object(map) = json {
        Ok(map.keys().cloned().collect())
    } else {
        Err(Error::new(
            ErrorKind::InvalidData,
            "Failed to extract keys. JSON object expected.",
        ))
    }
}

fn get_rs_files(dir: &str) -> Result<Vec<PathBuf>, io::Error> {
    let mut rs_files = Vec::new();
    collect_rs_files(Path::new(dir), &mut rs_files)?;
    Ok(rs_files)
}

fn collect_rs_files(dir: &Path, rs_files: &mut Vec<PathBuf>) -> Result<(), io::Error> {
    fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .try_fold((), |(), path| {
            if path.is_dir() {
                if !path.ends_with("src/bin") {
                    collect_rs_files(&path, rs_files)?;
                }
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                rs_files.push(path);
            }
            Ok(())
        })
}

fn collect_strings_from_file(dir: &Path, re: &Regex) -> Result<HashSet<String>, io::Error> {
    let content = fs::read_to_string(dir)?;

    let strings: HashSet<_> = re
        .captures_iter(&content)
        .filter_map(|cap| cap.get(1))
        .filter_map(|m| {
            let matched_string = m.as_str();
            let start = m.start() - 1;

            // Get the preceding three lines
            let preceding_lines: Vec<&str> = content[..start]
                .lines()
                .rev()
                .take(3)
                .map(str::trim)
                .collect();

            if preceding_lines.iter().any(|line| line.contains("text!(")) {
                return Some(matched_string.to_string());
            }

            // Check if the string is inside a dynamic context
            if preceding_lines
                .iter()
                .enumerate()
                .any(|(i, line)| i <= 2 && line.contains("write!("))
            {
                return None;
            }

            if preceding_lines
                .iter()
                .enumerate()
                .any(|(i, line)| i == 0 && line.contains(".format("))
            {
                return None;
            }

            if preceding_lines.iter().enumerate().any(|(i, line)| match i {
                0 => line.contains("format!"),
                1 => {
                    preceding_lines.first().is_some_and(|prev| prev.is_empty())
                        && line.contains("format!(")
                }
                _ => false,
            }) {
                return None;
            }

            Some(matched_string.to_string())
        })
        .collect();

    Ok(strings)
}
