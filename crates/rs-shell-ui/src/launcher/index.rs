use anyhow::Result;
use freedesktop_desktop_entry::DesktopEntry;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// A discovered application entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppEntry {
    /// Desktop file id (filename without .desktop).
    pub id: String,
    /// Display name.
    pub name: String,
    /// Command to execute (field codes stripped).
    pub exec: String,
    /// Optional icon name.
    pub icon: Option<String>,
    /// Optional generic name.
    pub generic_name: Option<String>,
    /// Keywords for search.
    pub keywords: Vec<String>,
    /// Whether the app should run in a terminal.
    pub terminal: bool,
    /// Path to the .desktop file.
    pub path: PathBuf,
}

/// Scan all XDG application directories for launchable apps.
pub fn scan_applications() -> Result<Vec<AppEntry>> {
    let mut seen = HashSet::new();
    let mut entries = Vec::new();
    let locales = locales();

    for dir in application_dirs() {
        for path in desktop_files(&dir) {
            let Some(id) = desktop_id(&path) else {
                continue;
            };
            if seen.contains(&id) {
                continue;
            }

            let Ok(entry) = DesktopEntry::from_path(path.clone(), Some(&locales)) else {
                continue;
            };
            if entry.type_().is_some_and(|type_| type_ != "Application") {
                continue;
            }
            if entry.no_display() || entry.hidden() {
                continue;
            }
            let Some(name) = entry.name(&locales).filter(|name| !name.is_empty()) else {
                continue;
            };
            let Some(exec) = entry
                .exec()
                .map(strip_field_codes)
                .filter(|exec| !exec.is_empty())
            else {
                continue;
            };

            seen.insert(id.clone());
            entries.push(AppEntry {
                id,
                name: name.into_owned(),
                exec,
                icon: entry.icon().map(str::to_string),
                generic_name: entry
                    .generic_name(&locales)
                    .filter(|value| !value.is_empty())
                    .map(|value| value.into_owned()),
                keywords: entry
                    .keywords(&locales)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|value| value.into_owned())
                    .filter(|value| !value.is_empty())
                    .collect(),
                terminal: entry.terminal(),
                path,
            });
        }
    }

    entries.sort_by(|a, b| a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()));
    Ok(entries)
}

/// Remove freedesktop field codes (%f, %F, %u, %U, %i, %c, %k) from an Exec line.
pub fn strip_field_codes(exec: &str) -> String {
    exec.split_whitespace()
        .filter(|part| !is_field_code_token(part))
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_field_code_token(part: &str) -> bool {
    let bytes = part.as_bytes();
    bytes.len() == 2
        && bytes[0] == b'%'
        && matches!(bytes[1], b'f' | b'F' | b'u' | b'U' | b'i' | b'c' | b'k')
}

fn locales() -> Vec<String> {
    let mut values = Vec::new();
    if let Ok(lang) = std::env::var("LANG") {
        values.push(lang);
    }
    if let Ok(languages) = std::env::var("LANGUAGES") {
        values.extend(
            languages
                .split(':')
                .filter(|value| !value.is_empty())
                .map(str::to_string),
        );
    }
    values
}

fn application_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(data_home) = std::env::var_os("XDG_DATA_HOME") {
        dirs.push(PathBuf::from(data_home).join("applications"));
    } else if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join(".local/share/applications"));
    }

    let data_dirs = std::env::var_os("XDG_DATA_DIRS")
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "/usr/local/share:/usr/share".to_string());
    dirs.extend(
        data_dirs
            .split(':')
            .filter(|value| !value.is_empty())
            .map(|dir| PathBuf::from(dir).join("applications")),
    );
    dirs
}

fn desktop_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(read_dir) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in read_dir.flatten() {
            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "desktop") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

fn desktop_id(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix(".desktop"))
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_field_code_tokens() {
        assert_eq!(strip_field_codes("code %F"), "code");
        assert_eq!(strip_field_codes("kitty +open %U"), "kitty +open");
        assert_eq!(strip_field_codes("foo %i %c"), "foo");
    }

    #[test]
    fn desktop_id_parses() {
        let path = PathBuf::from("/usr/share/applications/firefox.desktop");
        assert_eq!(desktop_id(&path), Some("firefox".to_string()));
    }
}
