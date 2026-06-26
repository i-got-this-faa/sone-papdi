pub mod index;
pub mod score;

pub use index::{scan_applications, AppEntry, strip_field_codes};
pub use score::{search, ScoredEntry};

use anyhow::{Context, Result};
use sone_papdi_config::LauncherConfig;
use std::process::Command;

/// Launcher state that holds indexed applications.
#[derive(Clone, Debug, Default)]
pub struct LauncherIndex {
    apps: Vec<AppEntry>,
    config: LauncherConfig,
}

impl LauncherIndex {
    /// Create a new launcher index with the given config.
    pub fn new(config: LauncherConfig) -> Self {
        Self {
            apps: Vec::new(),
            config,
        }
    }

    /// Rebuild the index from the filesystem.
    pub fn refresh(&mut self) -> Result<()> {
        self.apps = scan_applications()?;
        Ok(())
    }

    /// Search the index for the given query.
    pub fn search(&self, query: &str) -> Vec<ScoredEntry> {
        score::search(&self.apps, query, self.config.max_results)
    }

    /// Launch the given app entry.
    pub fn launch(&self, entry: &AppEntry) -> Result<()> {
        let exec = if entry.terminal {
            format!("{} {}", self.config.terminal_prefix, entry.exec)
        } else {
            entry.exec.clone()
        };
        let parts: Vec<&str> = exec.split_whitespace().collect();
        if parts.is_empty() {
            anyhow::bail!("empty exec line for {}", entry.name);
        }

        let mut cmd = Command::new(parts[0]);
        cmd.args(&parts[1..]);
        cmd.spawn()
            .with_context(|| format!("launching {} with {}", entry.name, parts.join(" ")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launcher_index_refresh_and_search() {
        let mut launcher = LauncherIndex::new(LauncherConfig::default());
        // Refresh may fail in environments without XDG dirs; allow it.
        if launcher.refresh().is_err() {
            return;
        }
        let results = launcher.search("firefox");
        // Only assert that search returns without panic; actual results depend on host.
        assert!(results.len() <= LauncherConfig::default().max_results);
    }
}
