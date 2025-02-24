use std::collections::HashSet;
use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};

use regex::Regex;
use serde_json::Value;

const FIXED_EXCLUDED_STRINGS: &[&str] = &[
    "en-US",
    "ko-KR",
    "statisticsChart-{}-{}-{}-{}-{}-{}",
    "Y-m-d H:i",
];

fn main() -> Result<(), io::Error> {
    let en_path = "langs/en-US.json";
    let ko_path = "langs/ko-KR.json";
    let _en_key_list = extract_keys_from_json(en_path)?;
    let _ko_key_list = extract_keys_from_json(ko_path)?;
    let rs_files = get_files_with_extension("src", "rs")?;
    let css_files = get_files_with_extension("static", "css")?;
    let css_classes_and_ids = extract_css_classes_and_ids(&css_files)?;

    let re = Regex::new(r#""([^"\\]*(\\.[^"\\]*)*)""#)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let mut all_strings: HashSet<String> = rs_files
        .into_iter()
        .map(|path| collect_strings_from_file(&path, &re))
        .flat_map(|result| result.into_iter().flatten())
        .collect::<HashSet<_>>();

    all_strings.retain(|s| {
        !FIXED_EXCLUDED_STRINGS
            .iter()
            .any(|&excluded| excluded == s.as_str())
            && !css_classes_and_ids
                .iter()
                .any(|class_or_id| class_or_id == s)
    });

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

fn get_files_with_extension(dir: &str, extension: &str) -> Result<Vec<PathBuf>, io::Error> {
    let mut files = Vec::new();
    collect_files_with_extension(Path::new(dir), &mut files, extension)?;
    Ok(files)
}

fn collect_files_with_extension(
    dir: &Path,
    files: &mut Vec<PathBuf>,
    extension: &str,
) -> Result<(), io::Error> {
    // Define paths to exclude
    let exclude_paths: HashSet<PathBuf> = vec![
        PathBuf::from("src/triage/policy/data.rs"),
        PathBuf::from("src/detection/mitre.rs"),
    ]
    .into_iter()
    .collect();

    fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .try_for_each(|path| {
            if path.is_dir() {
                if !path.ends_with("src/bin") {
                    collect_files_with_extension(&path, files, extension)?;
                }
            } else if path.extension().and_then(|ext| ext.to_str()) == Some(extension)
                && !exclude_paths.contains(&path)
            {
                files.push(path);
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

            if matched_string.chars().all(|c| !c.is_alphabetic())
                || (matches!(matched_string.chars().next(), Some('/' | '#'))
                    && matched_string.chars().nth(1).is_some_and(|c| c != ' '))
                || matched_string.contains("%Y")
                || matched_string
                    .chars()
                    .any(|c| ('\u{AC00}'..='\u{D7A3}').contains(&c))
            {
                return None;
            }

            let line_start = content[..start].rfind('\n').map_or(0, |pos| pos + 1);
            let line_end = content[start..]
                .find('\n')
                .map_or(content.len(), |pos| start + pos);
            let current_line = content[line_start..line_end].trim();

            if current_line.contains("expect(") || current_line.contains("feature =") {
                return None;
            }

            let preceding_lines: Vec<&str> = content[..start]
                .lines()
                .rev()
                .take(4)
                .map(str::trim)
                .collect();

            if preceding_lines
                .first()
                .is_some_and(|line| line.contains("text!("))
            {
                return Some(matched_string.to_string());
            }

            (!preceding_lines.iter().enumerate().any(|(i, line)| {
                line.contains("#[graphql(")
                    || (i == 0 && line.contains("type="))
                    || (i <= 1 && line.contains("anyhow!("))
                    || (i <= 2 && line.contains("write!("))
                    || (line.contains("format!(")
                        && (i == 0
                            || (i == 1
                                && preceding_lines.first().is_some_and(|prev| prev.is_empty()))
                            || (i == 2
                                && preceding_lines.get(1).is_some_and(|prev| prev.is_empty()))))
            }))
            .then(|| matched_string.to_string())
        })
        .collect();

    Ok(strings)
}

fn extract_css_classes_and_ids(css_file_paths: &[PathBuf]) -> Result<HashSet<String>, io::Error> {
    let class_re = Regex::new(r"(?:[a-zA-Z]+\.)?\.([a-zA-Z][a-zA-Z0-9_-]*)")
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let id_re = Regex::new(r"(?:[a-zA-Z]+#)?#([a-zA-Z][a-zA-Z0-9_-]*)")
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let classes_and_ids = css_file_paths
        .iter()
        .filter_map(|path| fs::read_to_string(path).ok())
        .flat_map(|content| {
            content
                .lines()
                .flat_map(|line| {
                    let mut combined_matches = Vec::new();

                    combined_matches.extend(
                        class_re
                            .captures_iter(line)
                            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_owned())),
                    );

                    combined_matches.extend(
                        id_re
                            .captures_iter(line)
                            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_owned())),
                    );

                    combined_matches
                })
                .collect::<Vec<String>>()
        })
        .collect::<HashSet<String>>();

    Ok(classes_and_ids)
}
