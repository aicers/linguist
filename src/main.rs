mod repo;
use std::collections::HashSet;
use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};

use clap::Parser;
use regex::Regex;
use repo::{validate_ssh_key_path, RepoManager};
use serde_json::Value;
use toml::Value as TomlValue;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(long)]
    ui_path: Option<PathBuf>,

    #[arg(long)]
    frontary_path: Option<PathBuf>,

    #[arg(long, value_name = "SSH_KEY")]
    ssh_key: Option<PathBuf>,
}

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

const AICE_WEB_URL: &str = "git@github.com:aicers/aice-web.git";
const FRONTARY_URL: &str = "https://github.com/aicers/frontary.git";
const UI_REPO_NAME: &str = "aice-web";
const FRONTARY_REPO_NAME: &str = "frontary";

fn main() -> Result<(), io::Error> {
    let args = Args::parse();

    // Validate SSH key if provided
    if let Some(ref ssh_key_path) = args.ssh_key {
        validate_ssh_key_path(ssh_key_path)
            .map_err(|e| io::Error::other(e.message().to_owned()))?;
    }

    let repo_manager = RepoManager::new(args.ssh_key.clone())
        .map_err(|e| io::Error::other(format!("Failed to create RepoManager: {e}")))?;

    log_repo_strategy(args.ui_path.as_ref(), args.frontary_path.as_ref());

    let ui_repo = prepare_repo(
        AICE_WEB_URL,
        args.ui_path.clone(),
        UI_REPO_NAME,
        &repo_manager,
    )?;

    let fr_repo = prepare_repo(
        FRONTARY_URL,
        args.frontary_path.clone(),
        FRONTARY_REPO_NAME,
        &repo_manager,
    )?;

    checkout_frontary(args.frontary_path.as_ref(), &ui_repo, &fr_repo)?;
    process_keys(&ui_repo, &fr_repo)?;
    Ok(())
}

fn log_repo_strategy(ui_path: Option<&PathBuf>, fr_path: Option<&PathBuf>) {
    match (ui_path, fr_path) {
        (None, None) => println!(
            "🔄 No local paths: will clone both '{UI_REPO_NAME}' and '{FRONTARY_REPO_NAME}'."
        ),
        (Some(path), None) => println!(
            "🔄 Using local {UI_REPO_NAME} at {}; will clone {FRONTARY_REPO_NAME}.",
            path.display()
        ),
        (None, Some(path)) => println!(
            "🔄 Will clone {UI_REPO_NAME}; using local {FRONTARY_REPO_NAME} at {}.",
            path.display()
        ),
        (Some(ui), Some(fr)) => println!(
            "🔄 Using local {UI_REPO_NAME} at {} and {FRONTARY_REPO_NAME} at {}.",
            ui.display(),
            fr.display()
        ),
    }
}

fn checkout_frontary(
    fr_local: Option<&PathBuf>,
    ui_repo: &Path,
    fr_repo: &Path,
) -> Result<(), io::Error> {
    if fr_local.is_none() {
        let tag = read_frontary_req(ui_repo)?;
        println!("🔀 Checking out frontary at commit: {tag}");
        RepoManager::checkout(fr_repo, &tag)
            .map_err(|e| io::Error::other(format!("Checkout failed: {e}")))?;
    }
    Ok(())
}

fn prepare_repo(
    repo_url: &str,
    override_path: Option<PathBuf>,
    name: &str,
    manager: &RepoManager,
) -> Result<PathBuf, io::Error> {
    if let Some(path) = override_path {
        if path.exists() {
            return Ok(path);
        }
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Local {name} repo not found at {}", path.display()),
        ));
    }

    println!("🛠️ Cloning repository: {repo_url}...");

    let cloned = manager
        .clone_repo(repo_url, name)
        .map_err(|e| io::Error::other(format!("Failed to clone {name}: {e}")))?;
    Ok(cloned)
}

fn process_keys(ui_repo: &Path, fr_repo: &Path) -> Result<(), io::Error> {
    // collect paths & files
    let en_path = ui_repo.join("langs/en-US.json");
    let ko_path = ui_repo.join("langs/ko-KR.json");
    let ui_files = get_files_with_extension(ui_repo.join("src"), "rs")?;
    let css_files = get_files_with_extension(ui_repo.join("static"), "css")?;
    let frontary_files = get_files_with_extension(fr_repo.join("src"), "rs")?;
    let css_ids = extract_css_classes_and_ids(&css_files)?;
    // JSON keys
    let en_keys = extract_keys_from_json(&en_path)?;
    let ko_keys = extract_keys_from_json(&ko_path)?;
    // regex for string literals
    let re = Regex::new(r#""([^"\\]*(\\.[^"\\]*)*)""#)
        .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;

    let mut ui_strings = ui_files
        .into_iter()
        .map(|p| collect_strings_from_file(&p, &re))
        .flat_map(Result::into_iter)
        .flatten()
        .collect::<HashSet<_>>();
    ui_strings.retain(|s| {
        !FIXED_EXCLUDED_STRINGS.iter().any(|&e| e == s) && !css_ids.iter().any(|id| id == s)
    });
    ui_strings.extend(FIXED_UI_KEY.iter().map(ToString::to_string));

    let mut frontary_strings = frontary_files
        .into_iter()
        .map(|p| extract_frontary_keys_from_file(&p, &re))
        .flat_map(Result::into_iter)
        .flatten()
        .collect::<HashSet<_>>();
    frontary_strings.extend(FIXED_FRONTARY_KEY.iter().map(ToString::to_string));

    let all_strings = ui_strings.union(&frontary_strings).cloned().collect();
    compare_keys("all_strings", &all_strings, "ko-KR.json", &ko_keys);
    compare_keys("all_strings", &all_strings, "en-US.json", &en_keys);
    compare_keys("ko-KR.json", &ko_keys, "en-US.json", &en_keys);
    Ok(())
}

fn read_frontary_req(ui_root: &Path) -> Result<String, io::Error> {
    let cargo_toml = ui_root.join("Cargo.toml");
    let toml_str = fs::read_to_string(&cargo_toml).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("Failed to read {}: {e}", cargo_toml.display()),
        )
    })?;

    let cargo: TomlValue = toml::from_str(&toml_str)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Invalid TOML: {e}")))?;

    if let Some(frontary) = cargo
        .get("dependencies")
        .and_then(|deps| deps.get("frontary"))
    {
        if let Some(version) = frontary.as_str() {
            return Ok(version.to_string());
        }
        if let Some(table) = frontary.as_table() {
            if let Some(tag) = table.get("tag").and_then(TomlValue::as_str) {
                return Ok(tag.to_string());
            }
            if let Some(rev) = table.get("rev").and_then(TomlValue::as_str) {
                return Ok(rev.to_string());
            }
            if let Some(ver) = table.get("version").and_then(TomlValue::as_str) {
                return Ok(ver.to_string());
            }
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "`frontary` dependency not found in Cargo.toml",
    ))
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

fn print_missing(
    from_name: &str,
    to_name: &str,
    from_set: &HashSet<String>,
    to_set: &HashSet<String>,
) {
    let missing = from_set
        .difference(to_set)
        .fold(String::new(), |mut acc, key| {
            acc.push_str("  - ");
            acc.push_str(key);
            acc.push('\n');
            acc
        });

    if missing.is_empty() {
        println!("No keys from `{from_name}` are missing in `{to_name}`.");
    } else {
        println!("Keys from `{from_name}` missing in `{to_name}`:\n{missing}");
    }
}

fn compare_keys(name1: &str, set1: &HashSet<String>, name2: &str, set2: &HashSet<String>) {
    println!("=== {name1} vs {name2} ===");

    // keys in set1 not in set2
    print_missing(name1, name2, set1, set2);

    // keys in set2 not in set1
    print_missing(name2, name1, set2, set1);

    println!();
}
