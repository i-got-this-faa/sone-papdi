use anyhow::{Context, Result};
use chrono::Utc;
use rs_shell_core::events::{ClipEntry, ClipEntryKind};
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;

/// Clipboard database handle.
pub struct Database {
    conn: Connection,
}

/// A new clipboard entry to be stored.
#[derive(Debug, Clone)]
pub struct NewClipEntry {
    pub content_hash: String,
    pub mime: String,
    pub kind: ClipEntryKind,
    pub preview: String,
    pub content: Option<String>,
    pub size_bytes: i64,
}

impl Database {
    /// Open the database, creating the schema if necessary.
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating database directory {}", parent.display()))?;
        }
        let conn = Connection::open(path)
            .with_context(|| format!("opening database at {}", path.display()))?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS entries (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              content_hash TEXT NOT NULL UNIQUE,
              kind TEXT NOT NULL,
              mime TEXT NOT NULL,
              preview TEXT NOT NULL,
              content TEXT,
              pinned INTEGER NOT NULL DEFAULT 0,
              created_at INTEGER NOT NULL,
              updated_at INTEGER NOT NULL,
              size_bytes INTEGER NOT NULL DEFAULT 0,
              deleted INTEGER NOT NULL DEFAULT 0
            );

            CREATE INDEX IF NOT EXISTS idx_entries_hash ON entries(content_hash);
            CREATE INDEX IF NOT EXISTS idx_entries_updated_at ON entries(updated_at DESC);
            CREATE INDEX IF NOT EXISTS idx_entries_pinned ON entries(pinned DESC, updated_at DESC);
            CREATE INDEX IF NOT EXISTS idx_entries_kind ON entries(kind);
            CREATE INDEX IF NOT EXISTS idx_entries_preview ON entries(preview);
            "#,
        )?;
        Ok(())
    }

    /// Insert or update a clipboard entry.
    pub fn upsert_entry(&self, entry: &NewClipEntry) -> Result<i64> {
        let now = Utc::now().timestamp();
        let kind = kind_to_str(&entry.kind);

        self.conn.execute(
            r#"
            INSERT INTO entries (
              content_hash, kind, mime, preview, content,
              created_at, updated_at, size_bytes, deleted
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, ?7, 0)
            ON CONFLICT(content_hash) DO UPDATE SET
              kind=excluded.kind,
              mime=excluded.mime,
              preview=excluded.preview,
              content=excluded.content,
              updated_at=excluded.updated_at,
              size_bytes=excluded.size_bytes,
              deleted=0
            "#,
            params![
                entry.content_hash,
                kind,
                entry.mime,
                entry.preview,
                entry.content,
                now,
                entry.size_bytes,
            ],
        )?;

        let id = self.conn.query_row(
            "SELECT id FROM entries WHERE content_hash = ?1",
            params![entry.content_hash],
            |row| row.get::<_, i64>(0),
        )?;
        Ok(id)
    }

    /// Return a paginated list of non-deleted entries.
    pub fn list_entries(&self, query: &str, limit: usize, offset: usize) -> Result<Vec<ClipEntry>> {
        let mut sql = String::from(
            "SELECT id, content_hash, kind, mime, preview, content, pinned, created_at, updated_at, size_bytes \
             FROM entries WHERE deleted = 0",
        );

        let has_query = !query.trim().is_empty();
        if has_query {
            sql.push_str(" AND preview LIKE ?1");
        }
        sql.push_str(" ORDER BY pinned DESC, updated_at DESC");

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = if has_query {
            let pattern = format!("%{}%", query.trim());
            stmt.query_map(params![pattern], row_to_entry)?
                .collect::<rusqlite::Result<Vec<_>>>()?
        } else {
            stmt.query_map([], row_to_entry)?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };

        // Apply limit/offset in Rust to keep query simple.
        Ok(rows.into_iter().skip(offset).take(limit).collect())
    }

    /// Count non-deleted entries.
    pub fn count_entries(&self) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM entries WHERE deleted = 0", [], |row| {
                row.get(0)
            })?;
        Ok(count.max(0) as usize)
    }

    /// Get a single entry by id.
    pub fn get_entry(&self, id: i64) -> Result<Option<ClipEntry>> {
        self.conn
            .query_row(
                "SELECT id, content_hash, kind, mime, preview, content, pinned, created_at, updated_at, size_bytes \
                 FROM entries WHERE id = ?1 AND deleted = 0",
                params![id],
                row_to_entry,
            )
            .optional()
            .map_err(Into::into)
    }

    /// Soft-delete an entry.
    pub fn delete_entry(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE entries SET deleted = 1, updated_at = ?2 WHERE id = ?1",
            params![id, Utc::now().timestamp()],
        )?;
        Ok(())
    }

    /// Delete all non-pinned entries older than the given number of days.
    pub fn delete_unpinned_older_than_days(&self, days: u32) -> Result<usize> {
        if days == 0 {
            return Ok(0);
        }
        let cutoff = Utc::now().timestamp() - i64::from(days) * 86_400;
        let deleted = self.conn.execute(
            "UPDATE entries SET deleted = 1, updated_at = ?2 WHERE deleted = 0 AND pinned = 0 AND updated_at < ?1",
            params![cutoff, Utc::now().timestamp()],
        )?;
        Ok(deleted)
    }

    /// Toggle the pinned state of an entry.
    pub fn set_pinned(&self, id: i64, pinned: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE entries SET pinned = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, pinned, Utc::now().timestamp()],
        )?;
        Ok(())
    }

    /// Mark an entry as used (copied).
    pub fn touch_used(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE entries SET updated_at = ?2 WHERE id = ?1",
            params![id, Utc::now().timestamp()],
        )?;
        Ok(())
    }
}

fn kind_to_str(kind: &ClipEntryKind) -> &'static str {
    match kind {
        ClipEntryKind::Text => "text",
        ClipEntryKind::Link => "link",
        ClipEntryKind::Color => "color",
        ClipEntryKind::Image => "image",
        ClipEntryKind::File => "file",
        ClipEntryKind::Secret => "secret",
    }
}

fn str_to_kind(s: &str) -> ClipEntryKind {
    match s {
        "text" => ClipEntryKind::Text,
        "link" => ClipEntryKind::Link,
        "color" => ClipEntryKind::Color,
        "image" => ClipEntryKind::Image,
        "file" => ClipEntryKind::File,
        "secret" => ClipEntryKind::Secret,
        _ => ClipEntryKind::Text,
    }
}

fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<ClipEntry> {
    let created_at: i64 = row.get("created_at")?;
    let updated_at: i64 = row.get("updated_at")?;
    Ok(ClipEntry {
        id: row.get("id")?,
        mime: row.get("mime")?,
        kind: str_to_kind(&row.get::<_, String>("kind")?),
        preview: row.get("preview")?,
        content: row.get("content")?,
        pinned: row.get::<_, i64>("pinned")? != 0,
        created_at: chrono::DateTime::from_timestamp(created_at, 0).unwrap_or_else(|| chrono::Utc::now()),
        updated_at: chrono::DateTime::from_timestamp(updated_at, 0).unwrap_or_else(|| chrono::Utc::now()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db() -> Database {
        use std::time::{SystemTime, UNIX_EPOCH};
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "rs-shell-clipboard-test-{}-{unique}.sqlite",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        Database::open(&path).unwrap()
    }

    fn text_entry(text: &str) -> NewClipEntry {
        NewClipEntry {
            content_hash: blake3::hash(text.as_bytes()).to_string(),
            mime: "text/plain".into(),
            kind: ClipEntryKind::Text,
            preview: text.chars().take(80).collect(),
            content: Some(text.into()),
            size_bytes: text.len() as i64,
        }
    }

    #[test]
    fn upsert_and_list() {
        let db = temp_db();
        let id = db.upsert_entry(&text_entry("hello world")).unwrap();
        let entries = db.list_entries("", 10, 0).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, id);
        assert_eq!(entries[0].preview, "hello world");
    }

    #[test]
    fn search_filters_entries() {
        let db = temp_db();
        db.upsert_entry(&text_entry("alpha")).unwrap();
        db.upsert_entry(&text_entry("beta")).unwrap();
        let entries = db.list_entries("alpha", 10, 0).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].preview, "alpha");
    }

    #[test]
    fn delete_and_pin() {
        let db = temp_db();
        let id = db.upsert_entry(&text_entry("pin me")).unwrap();
        db.set_pinned(id, true).unwrap();
        let entry = db.get_entry(id).unwrap().unwrap();
        assert!(entry.pinned);

        db.delete_entry(id).unwrap();
        assert!(db.get_entry(id).unwrap().is_none());
    }

    #[test]
    fn cleanup_old_unpinned() {
        let db = temp_db();
        let now = Utc::now().timestamp();
        let old = now - 3 * 86_400;

        for (hash, title, updated_at, pinned) in [
            ("old-unpinned", "old unpinned", old, 0),
            ("old-pinned", "old pinned", old, 1),
            ("recent-unpinned", "recent unpinned", now, 0),
        ] {
            db.conn
                .execute(
                    "INSERT INTO entries (content_hash, kind, mime, preview, content, pinned, created_at, updated_at, size_bytes) \
                     VALUES (?1, 'text', 'text/plain', ?2, ?2, ?3, ?4, ?4, ?5)",
                    params![hash, title, pinned, updated_at, title.len() as i64],
                )
                .unwrap();
        }

        let deleted = db.delete_unpinned_older_than_days(1).unwrap();
        assert_eq!(deleted, 1);
        assert_eq!(db.count_entries().unwrap(), 2);
    }
}
