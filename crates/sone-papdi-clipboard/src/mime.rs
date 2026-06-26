use sone_papdi_core::events::ClipEntryKind;

/// Classify a clipboard payload based on MIME type and content.
pub fn classify(mime: &str, content: &[u8]) -> (ClipEntryKind, String) {
    let mime_lower = mime.to_lowercase();

    if mime_lower.starts_with("image/") {
        return (ClipEntryKind::Image, "<image>".into());
    }

    if mime_lower == "text/uri-list" {
        return (ClipEntryKind::File, preview_text(content, 80));
    }

    let text = String::from_utf8_lossy(content);
    let trimmed = text.trim();

    if mime_lower.starts_with("text/") || mime_lower.is_empty() {
        if looks_like_color(trimmed) {
            return (ClipEntryKind::Color, trimmed.to_string());
        }
        if looks_like_url(trimmed) {
            return (ClipEntryKind::Link, preview_text(content, 80));
        }
        return (ClipEntryKind::Text, preview_text(content, 80));
    }

    (ClipEntryKind::Secret, preview_text(content, 80))
}

fn looks_like_color(s: &str) -> bool {
    s.starts_with('#') && (s.len() == 4 || s.len() == 7 || s.len() == 9)
        || s.starts_with("rgb")
        || s.starts_with("hsl")
        || s.starts_with("oklch")
}

fn looks_like_url(s: &str) -> bool {
    s.starts_with("http://")
        || s.starts_with("https://")
        || s.starts_with("ftp://")
        || s.starts_with("file://")
}

fn preview_text(content: &[u8], max_len: usize) -> String {
    String::from_utf8_lossy(content)
        .chars()
        .take(max_len)
        .collect::<String>()
        .replace('\n', " ")
        .replace('\r', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_text() {
        let (kind, preview) = classify("text/plain", b"hello world");
        assert_eq!(kind, ClipEntryKind::Text);
        assert_eq!(preview, "hello world");
    }

    #[test]
    fn classify_link() {
        let (kind, _) = classify("text/plain", b"https://example.com");
        assert_eq!(kind, ClipEntryKind::Link);
    }

    #[test]
    fn classify_color() {
        let (kind, _) = classify("text/plain", b"#7dcfff");
        assert_eq!(kind, ClipEntryKind::Color);
    }

    #[test]
    fn classify_image() {
        let (kind, preview) = classify("image/png", b"fake");
        assert_eq!(kind, ClipEntryKind::Image);
        assert_eq!(preview, "<image>");
    }
}
