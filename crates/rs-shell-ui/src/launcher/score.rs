use super::index::AppEntry;

/// A scored search result.
#[derive(Clone, Debug)]
pub struct ScoredEntry {
    pub entry: AppEntry,
    pub score: i64,
}

/// Search indexed apps by query and return ranked results.
pub fn search(apps: &[AppEntry], query: &str, max_results: usize) -> Vec<ScoredEntry> {
    let query = query.to_ascii_lowercase();
    let mut results: Vec<ScoredEntry> = apps
        .iter()
        .filter_map(|entry| {
            score_entry(entry, &query).map(|score| ScoredEntry {
                entry: entry.clone(),
                score,
            })
        })
        .collect();

    results.sort_by(|a, b| b.score.cmp(&a.score));
    results.into_iter().take(max_results).collect()
}

fn score_entry(entry: &AppEntry, query: &str) -> Option<i64> {
    let name_lower = entry.name.to_ascii_lowercase();

    if name_lower == query {
        return Some(1000);
    }
    if name_lower.starts_with(query) {
        return Some(900 - name_lower.len() as i64);
    }
    if name_lower.contains(query) {
        return Some(700);
    }

    let mut score = 0i64;

    // Subsequence match in name.
    if is_subsequence(&name_lower, query) {
        score += 500;
    }

    // Match in generic name.
    if let Some(generic) = &entry.generic_name {
        let generic_lower = generic.to_ascii_lowercase();
        if generic_lower.contains(query) {
            score += 300;
        } else if is_subsequence(&generic_lower, query) {
            score += 150;
        }
    }

    // Keyword matches.
    for keyword in &entry.keywords {
        let kw_lower = keyword.to_ascii_lowercase();
        if kw_lower == query {
            score += 250;
        } else if kw_lower.starts_with(query) {
            score += 150;
        } else if kw_lower.contains(query) {
            score += 75;
        } else if is_subsequence(&kw_lower, query) {
            score += 40;
        }
    }

    if score > 0 {
        Some(score)
    } else {
        None
    }
}

fn is_subsequence(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    let mut it = needle.chars();
    let mut current = it.next().unwrap();
    for ch in haystack.chars() {
        if ch == current {
            match it.next() {
                Some(next) => current = next,
                None => return true,
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn app(name: &str, generic: Option<&str>, keywords: &[&str]) -> AppEntry {
        AppEntry {
            id: name.to_ascii_lowercase().replace(' ', "-"),
            name: name.into(),
            exec: name.into(),
            icon: None,
            generic_name: generic.map(str::to_string),
            keywords: keywords.iter().map(|s| s.to_string()).collect(),
            terminal: false,
            path: PathBuf::from("/dev/null"),
        }
    }

    #[test]
    fn exact_match_scores_highest() {
        let apps = vec![
            app("Firefox", None, &[]),
            app("Firebird", None, &[]),
        ];
        let results = search(&apps, "firefox", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.name, "Firefox");
    }

    #[test]
    fn subsequence_match() {
        let apps = vec![app("Visual Studio Code", Some("editor"), &["code", "ide"])];
        let results = search(&apps, "vsc", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.name, "Visual Studio Code");
    }
}
