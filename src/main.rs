mod repo;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::process;

use regex::Regex;
use repo::{setup_ssh_agent, RepoManager};
use serde_json::Value;

const FIXED_EXCLUDED_STRINGS: &[&str] = &[
    "&nbsp;",
    "\\t",
    "Content-Type",
    "DCE/RPC Blocklist",
    "DNS Blocklist",
    "FTP Blocklist",
    "FTP Brute Force",
    "FTP Plain Text",
    "HTTP Blocklist",
    "Kerberos Blocklist",
    "LDAP Blocklist",
    "LDAP Brute Force",
    "LDAP Plain Text",
    "Locky Ransomware",
    "MQTT Blocklist",
    "Multi-host Port Scan",
    "NFS Blocklist",
    "NTLM Blocklist",
    "Port Scan",
    "RDP Blocklist",
    "SMTP Blocklist",
    "SMB Blocklist",
    "SSH Blocklist",
    "TLS Blocklist",
    "Y-m-d H:i",
    "account",
    "allowlist",
    "application/json",
    "blocklist",
    "customer",
    "en-US",
    "ko-KR",
    "node",
    "sampling policy",
    "statisticsChart-{}-{}-{}-{}-{}-{}",
    "text",
    "triage policy",
    "trusted domains",
];

const FIXED_FRONTARY_KEY: &[&str] = &[
    "(Input Example: 192.168.1.100 ~ 192.168.1.200)",
    "(Input Example: 192.168.10.0/24)",
    "Add",
    "Add a network",
    "Add another condition",
    "Comparison",
    "If you want to change your password, input a new one.",
    "Invalid GraphQL query",
    "Invalid GraphQL response",
    "Invalid IP address",
    "Invalid input",
    "Invalid input (valid examples: 10.1.1.1 ~ 10.1.1.20)",
    "Invalid input (valid examples: 10.84.1.7, 10.1.1.1 ~ 10.1.1.20, 192.168.10.0/24)",
    "Multiple IP addresses possible",
    "Multiple inputs possible (valid examples: 10.84.1.7, 10.1.1.1 ~ 10.1.1.20, 192.168.10.0/24)",
    "No success HTTPS status code",
    "Required",
    "The input already exists.",
    "The maximum number of input was reached.",
    "This field is required.",
    "Type",
    "Unauthorized",
    "Unknown error",
    "Wrong input",
    "Your password is too short.",
    "Your password must contain at least one lowercase alphabet.",
    "Your password must contain at least one number.",
    "Your password must contain at least one special character.",
    "Your password must contain at least one uppercase alphabet.",
    "Your password must not constain any spaces.",
    "Your password must not contain any control characters.",
    "Your password must not contain consecutive repeating characters.",
    "Your password must not contain more than 3 adjacent keyboard characters.",
    "no spaces, more than 7 characters, at least one number/uppercase/lowercase/special characters",
    "no spaces, more than 8 characters, at least one number/uppercase/lowercase/special characters, no consecutive repetition, and less than 4 adjacent keyboard characters"
];

const FIXED_UI_KEY: &[&str] = &[
    "1 hour",
    "1 min.",
    "10 min.",
    "10 minutes",
    "15 minutes",
    "2 days",
    "2 hours",
    "2 weeks",
    "3 min.",
    "30 min.",
    "30 minutes",
    "30 sec.",
    "5 min.",
    "5 minutes",
    "6 hours",
    "DNS",
    "Entire",
    "Events",
    "PDF",
    "RDP",
    "SSH",
    "Save FTP Files",
    "Save HTTP Files",
    "Save Packets",
    "Save SMTP Files",
    "Session",
    "Semi-supervised Learning",
    "System Administrator",
    "Token",
    "URL",
    "Unsupervised Learning",
    "Whitelist",
];

fn main() -> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path-to-ssh-key>", args[0]);
        process::exit(1);
    }

    let ssh_key_path = PathBuf::from(&args[1]);

    if !ssh_key_path.exists() {
        eprintln!("Error: SSH key not found at {ssh_key_path:?}");
        process::exit(1);
    }

    let repo_manager = RepoManager::new().map_err(|e| {
        eprintln!("Error creating temp directory: {e}");
        e
    })?;

    let frontary_url = "https://github.com/aicers/frontary.git";
    let ui_url = "git@github.com:aicers/aice-web.git";

    if let Err(e) = setup_ssh_agent(&ssh_key_path) {
        eprintln!("❌ Failed to set up SSH agent: {e}");
        return Err(io::Error::new(ErrorKind::Other, "SSH setup failed"));
    }

    println!("🛠️ Cloning repository: {frontary_url}...");
    if let Err(e) = repo_manager.clone_repo(frontary_url, "frontary") {
        eprintln!("Failed to clone frontary: {e}");
        return Err(io::Error::new(
            ErrorKind::Other,
            format!("❌ Failed to clone frontary: {e}"),
        ));
    }
    println!("🛠️ Cloning repository: {ui_url}...");
    if let Err(e) = repo_manager.clone_repo(ui_url, "aice-web") {
        eprintln!("Failed to clone aice-web: {e}");
        return Err(io::Error::new(
            ErrorKind::Other,
            format!("❌ Failed to clone aice-web: {e}"),
        ));
    }

    let temp_path = repo_manager.temp_dir.path();
    // Update file paths dynamically using `temp_path`
    let en_path = temp_path.join("aice-web/langs/en-US.json");
    let ko_path = temp_path.join("aice-web/langs/ko-KR.json");
    let ui_files = get_files_with_extension(temp_path.join("aice-web/src"), "rs")?;
    let css_files = get_files_with_extension(temp_path.join("aice-web/static"), "css")?;
    let frontary_file_paths = get_files_with_extension(temp_path.join("frontary"), "rs")?;
    let css_classes_and_ids = extract_css_classes_and_ids(&css_files)?;

    let _en_key_list = extract_keys_from_json(&en_path)?;
    let _ko_key_list = extract_keys_from_json(&ko_path)?;

    let re = Regex::new(r#""([^"\\]*(\\.[^"\\]*)*)""#)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let mut ui_strings: HashSet<String> = ui_files
        .into_iter()
        .map(|path| collect_strings_from_file(&path, &re))
        .flat_map(|result| result.into_iter().flatten())
        .collect::<HashSet<_>>();

    ui_strings.retain(|s| {
        !FIXED_EXCLUDED_STRINGS
            .iter()
            .any(|&excluded| excluded == s.as_str())
            && !css_classes_and_ids
                .iter()
                .any(|class_or_id| class_or_id == s)
    });

    ui_strings.extend(FIXED_UI_KEY.iter().map(ToString::to_string));

    let mut frontary_strings: HashSet<String> = frontary_file_paths
        .into_iter()
        .map(|path| extract_frontary_keys_from_file(&path, &re))
        .flat_map(|result| result.into_iter().flatten())
        .collect();

    frontary_strings.extend(FIXED_FRONTARY_KEY.iter().map(ToString::to_string));

    let _all_strings: HashSet<String> = ui_strings.union(&frontary_strings).cloned().collect();

    Ok(())
}

fn extract_keys_from_json<P: AsRef<Path>>(path: P) -> Result<HashSet<String>, io::Error> {
    let content = fs::read_to_string(path.as_ref())
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

fn get_files_with_extension<P: AsRef<Path>>(
    dir: P,
    extension: &str,
) -> Result<Vec<PathBuf>, io::Error> {
    let mut files = Vec::new();
    collect_files_with_extension(dir.as_ref(), &mut files, extension)?;
    Ok(files)
}

fn collect_files_with_extension(
    dir: &Path,
    files: &mut Vec<PathBuf>,
    extension: &str,
) -> Result<(), io::Error> {
    //Define paths to exclude
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
                && !exclude_paths.iter().any(|p| path.ends_with(p))
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
                || matched_string.starts_with("report-")
                || matched_string.len() == 1
            {
                return None;
            }

            let line_start = content[..start].rfind('\n').map_or(0, |pos| pos + 1);
            let line_end = content[start..]
                .find('\n')
                .map_or(content.len(), |pos| start + pos);
            let current_line = content[line_start..line_end].trim();

            if current_line.contains("expect(")
                || current_line.contains("feature =")
                || current_line.contains("#[serde(rename =")
                || current_line.contains("#[strum(serialize =")
            {
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

fn extract_frontary_keys_from_file(path: &Path, re: &Regex) -> Result<HashSet<String>, io::Error> {
    let content = fs::read_to_string(path)?;

    let keys: HashSet<_> = re
        .captures_iter(&content)
        .filter_map(|cap| cap.get(1))
        .filter_map(|m| {
            let matched_string = m.as_str();
            let start = m.start() - 1;

            let preceding_lines: Vec<&str> = content[..start]
                .lines()
                .rev()
                .take(4)
                .map(str::trim)
                .collect();

            preceding_lines
                .iter()
                .enumerate()
                .any(|(i, line)| {
                    (i == 0 && line.contains("ViewString::Key"))
                        || (line.contains("text!")
                            && (i == 0
                                || (i > 0
                                    && preceding_lines
                                        .iter()
                                        .find(|&&l| !l.is_empty())
                                        .is_some_and(|prev| prev.contains("ctx.props()")))))
                })
                .then(|| matched_string.to_string())
        })
        .collect();

    Ok(keys)
}
