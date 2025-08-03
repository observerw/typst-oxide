use crate::parser::models::{Label, ParsedFile, Wikilink};
use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension, params};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Index {
    conn: Connection,
    root: PathBuf,
}

impl Index {
    pub fn new(root: &Path) -> Result<Self> {
        let db_path = root.join(".pkm-cache.db");
        let conn = Connection::open(db_path)?;

        let cache = Index {
            conn,
            root: root.to_path_buf(),
        };

        cache.initialize_tables()?;
        Ok(cache)
    }

    fn initialize_tables(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS files (
                id INTEGER PRIMARY KEY,
                path TEXT UNIQUE NOT NULL,
                created_at DATETIME,
                modified_at DATETIME,
                last_parsed DATETIME
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS metadata (
                id INTEGER PRIMARY KEY,
                file_id INTEGER,
                key TEXT NOT NULL,
                value TEXT,
                FOREIGN KEY (file_id) REFERENCES files(id)
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS wikilinks (
                id INTEGER PRIMARY KEY,
                file_id INTEGER,
                target TEXT NOT NULL,
                alias TEXT,
                label TEXT,
                line INTEGER,
                column INTEGER,
                FOREIGN KEY (file_id) REFERENCES files(id)
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS labels (
                id INTEGER PRIMARY KEY,
                file_id INTEGER,
                name TEXT NOT NULL,
                line INTEGER,
                column INTEGER,
                is_implicit BOOLEAN,
                FOREIGN KEY (file_id) REFERENCES files(id)
            )",
            [],
        )?;

        // Create indexes for better performance
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_files_path ON files(path)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_metadata_file_id ON metadata(file_id)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wikilinks_file_id ON wikilinks(file_id)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wikilinks_target ON wikilinks(target)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_labels_file_id ON labels(file_id)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_labels_name ON labels(name)",
            [],
        )?;

        Ok(())
    }

    pub fn store_file(&mut self, file_path: &Path, parsed: &ParsedFile) -> Result<()> {
        let relative_path = self.get_relative_path(file_path)?;
        let metadata = std::fs::metadata(file_path)?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let tx = self.conn.transaction()?;

        // Insert or update file record
        tx.execute(
            "INSERT OR REPLACE INTO files (path, created_at, modified_at, last_parsed)
             VALUES (?, ?, ?, ?)",
            params![
                relative_path.to_str().context("Invalid UTF-8 in path")?,
                metadata
                    .created()
                    .ok()
                    .map(|t| t.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64),
                metadata
                    .modified()
                    .ok()
                    .map(|t| t.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64),
                now
            ],
        )?;

        let file_id: i64 = tx.last_insert_rowid();

        // Clear existing metadata, wikilinks, and labels
        tx.execute("DELETE FROM metadata WHERE file_id = ?", [file_id])?;
        tx.execute("DELETE FROM wikilinks WHERE file_id = ?", [file_id])?;
        tx.execute("DELETE FROM labels WHERE file_id = ?", [file_id])?;

        // Insert metadata
        if let Some(title) = &parsed.metadata.title {
            tx.execute(
                "INSERT INTO metadata (file_id, key, value) VALUES (?, ?, ?)",
                params![file_id, "title", title],
            )?;
        }

        for tag in &parsed.metadata.tags {
            tx.execute(
                "INSERT INTO metadata (file_id, key, value) VALUES (?, ?, ?)",
                params![file_id, "tags", tag],
            )?;
        }

        for alias in &parsed.metadata.alias {
            tx.execute(
                "INSERT INTO metadata (file_id, key, value) VALUES (?, ?, ?)",
                params![file_id, "alias", alias],
            )?;
        }

        // Insert custom metadata
        for (key, value) in &parsed.metadata.custom {
            tx.execute(
                "INSERT INTO metadata (file_id, key, value) VALUES (?, ?, ?)",
                params![file_id, key, value.to_string()],
            )?;
        }

        // Insert wikilinks
        for wikilink in &parsed.wikilinks {
            tx.execute(
                "INSERT INTO wikilinks (file_id, target, alias, label, line, column)
                 VALUES (?, ?, ?, ?, ?, ?)",
                params![
                    file_id,
                    wikilink.target,
                    wikilink.alias,
                    wikilink.label,
                    wikilink.line as i64,
                    wikilink.column as i64
                ],
            )?;
        }

        // Insert labels
        for label in &parsed.labels {
            tx.execute(
                "INSERT INTO labels (file_id, name, line, column)
                 VALUES (?, ?, ?, ?)",
                params![file_id, label.name, label.line as i64, label.column as i64],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn get_file(&self, file_path: &Path) -> Result<Option<ParsedFile>> {
        let relative_path = self.get_relative_path(file_path)?;

        let file_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM files WHERE path = ?",
                [relative_path.to_str().context("Invalid UTF-8 in path")?],
                |row| row.get(0),
            )
            .optional()?;

        let Some(file_id) = file_id else {
            return Ok(None);
        };

        let mut metadata = crate::parser::models::Metadata::default();

        // Get metadata
        let mut stmt = self
            .conn
            .prepare("SELECT key, value FROM metadata WHERE file_id = ?")?;
        let metadata_rows = stmt.query_map([file_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for row in metadata_rows {
            let (key, value) = row?;
            match key.as_str() {
                "title" => metadata.title = Some(value),
                "tags" => metadata.tags.push(value),
                "alias" => metadata.alias.push(value),
                _ => {
                    if let Ok(json_value) = serde_json::from_str(&value) {
                        metadata.custom.insert(key, json_value);
                    }
                }
            }
        }

        // Get wikilinks
        let mut wikilinks = Vec::new();
        let mut stmt = self.conn.prepare(
            "SELECT target, alias, label, line, column FROM wikilinks WHERE file_id = ?",
        )?;
        let wikilink_rows = stmt.query_map([file_id], |row| {
            Ok(Wikilink {
                target: row.get(0)?,
                alias: row.get(1)?,
                label: row.get(2)?,
                line: row.get::<_, i64>(3)? as usize,
                column: row.get::<_, i64>(4)? as usize,
            })
        })?;

        for wikilink in wikilink_rows {
            wikilinks.push(wikilink?);
        }

        // Get labels
        let mut labels = Vec::new();
        let mut stmt = self
            .conn
            .prepare("SELECT name, line, column FROM labels WHERE file_id = ?")?;
        let label_rows = stmt.query_map([file_id], |row| {
            Ok(Label {
                name: row.get(0)?,
                line: row.get::<_, i64>(1)? as usize,
                column: row.get::<_, i64>(2)? as usize,
            })
        })?;

        for label in label_rows {
            labels.push(label?);
        }

        Ok(Some(ParsedFile {
            path: file_path.to_path_buf(),
            metadata,
            wikilinks,
            labels,
        }))
    }

    pub fn get_backward_links(&self, target_file: &Path) -> Result<Vec<(PathBuf, Wikilink)>> {
        let target_name = target_file
            .file_stem()
            .and_then(|s| s.to_str())
            .context("Invalid file name")?;

        let mut stmt = self.conn.prepare(
            "SELECT f.path, w.target, w.alias, w.label, w.line, w.column
             FROM wikilinks w
             JOIN files f ON w.file_id = f.id
             WHERE w.target = ?",
        )?;

        let rows = stmt.query_map([target_name], |row| {
            let relative_path: String = row.get(0)?;
            let full_path = self.root.join(relative_path);
            let wikilink = Wikilink {
                target: row.get(1)?,
                alias: row.get(2)?,
                label: row.get(3)?,
                line: row.get::<_, i64>(4)? as usize,
                column: row.get::<_, i64>(5)? as usize,
            };
            Ok((full_path, wikilink))
        })?;

        let mut backlinks = Vec::new();
        for row in rows {
            backlinks.push(row?);
        }

        Ok(backlinks)
    }

    pub fn get_forward_links(&self, file_path: &Path) -> Result<Vec<Wikilink>> {
        let relative_path = self.get_relative_path(file_path)?;

        let file_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM files WHERE path = ?",
                [relative_path.to_str().context("Invalid UTF-8 in path")?],
                |row| row.get(0),
            )
            .optional()?;

        let Some(file_id) = file_id else {
            return Ok(Vec::new());
        };

        let mut stmt = self.conn.prepare(
            "SELECT target, alias, label, line, column FROM wikilinks WHERE file_id = ?",
        )?;

        let rows = stmt.query_map([file_id], |row| {
            Ok(Wikilink {
                target: row.get(0)?,
                alias: row.get(1)?,
                label: row.get(2)?,
                line: row.get::<_, i64>(3)? as usize,
                column: row.get::<_, i64>(4)? as usize,
            })
        })?;

        let mut wikilinks = Vec::new();
        for wikilink in rows {
            wikilinks.push(wikilink?);
        }

        Ok(wikilinks)
    }

    pub fn get_all_metadata(&self) -> Result<Vec<(String, String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT f.path, m.key, m.value 
             FROM metadata m 
             JOIN files f ON m.file_id = f.id
             ORDER BY f.path, m.key",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;

        let mut metadata = Vec::new();
        for row in rows {
            metadata.push(row?);
        }

        Ok(metadata)
    }

    pub fn remove_file(&self, file_path: &Path) -> Result<()> {
        let relative_path = self.get_relative_path(file_path)?;

        self.conn.execute(
            "DELETE FROM files WHERE path = ?",
            [relative_path.to_str().context("Invalid UTF-8 in path")?],
        )?;

        Ok(())
    }

    fn get_relative_path(&self, file_path: &Path) -> Result<PathBuf> {
        file_path
            .strip_prefix(&self.root)
            .map(|p| p.to_path_buf())
            .with_context(|| format!("File {} is not in workspace", file_path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::models::{Label, Metadata, Wikilink};
    use tempfile::tempdir;

    #[test]
    fn test_cache_initialization() {
        let temp_dir = tempdir().unwrap();
        let cache = Index::new(temp_dir.path()).unwrap();

        // Should not panic
        drop(cache);
    }

    #[test]
    fn test_store_and_retrieve_file() {
        let temp_dir = tempdir().unwrap();
        let mut cache = Index::new(temp_dir.path()).unwrap();

        let file_path = temp_dir.path().join("test.typ");
        std::fs::write(&file_path, "test content").unwrap();

        let parsed = ParsedFile {
            path: file_path.clone(),
            metadata: Metadata {
                title: Some("Test Note".to_string()),
                tags: vec!["tag1".to_string(), "tag2".to_string()],
                alias: vec!["alias1".to_string()],
                custom: std::collections::HashMap::new(),
            },
            wikilinks: vec![Wikilink {
                target: "other".to_string(),
                alias: Some("Other Note".to_string()),
                label: None,
                line: 1,
                column: 1,
            }],
            labels: vec![Label {
                name: "test-label".to_string(),
                line: 2,
                column: 5,
            }],
        };

        cache.store_file(&file_path, &parsed).unwrap();

        let retrieved = cache.get_file(&file_path).unwrap().unwrap();
        assert_eq!(retrieved.metadata.title, Some("Test Note".to_string()));
        assert_eq!(retrieved.wikilinks.len(), 1);
        assert_eq!(retrieved.labels.len(), 1);
    }

    #[test]
    fn test_backlinks() {
        let temp_dir = tempdir().unwrap();
        let mut cache = Index::new(temp_dir.path()).unwrap();

        let file1_path = temp_dir.path().join("file1.typ");
        let file2_path = temp_dir.path().join("file2.typ");

        std::fs::write(&file1_path, "content1").unwrap();
        std::fs::write(&file2_path, "content2").unwrap();

        let parsed1 = ParsedFile {
            path: file1_path.clone(),
            metadata: Metadata::default(),
            wikilinks: vec![Wikilink {
                target: "file2".to_string(),
                alias: None,
                label: None,
                line: 1,
                column: 1,
            }],
            labels: vec![],
        };

        let parsed2 = ParsedFile {
            path: file2_path.clone(),
            metadata: Metadata::default(),
            wikilinks: vec![],
            labels: vec![],
        };

        cache.store_file(&file1_path, &parsed1).unwrap();
        cache.store_file(&file2_path, &parsed2).unwrap();

        let backlinks = cache.get_backward_links(&file2_path).unwrap();
        assert_eq!(backlinks.len(), 1);
        assert_eq!(backlinks[0].1.target, "file2");
    }

    #[test]
    fn test_remove_file() {
        let temp_dir = tempdir().unwrap();
        let mut cache = Index::new(temp_dir.path()).unwrap();

        let file_path = temp_dir.path().join("test.typ");
        std::fs::write(&file_path, "test content").unwrap();

        let parsed = ParsedFile {
            path: file_path.clone(),
            metadata: Metadata::default(),
            wikilinks: vec![],
            labels: vec![],
        };

        cache.store_file(&file_path, &parsed).unwrap();
        assert!(cache.get_file(&file_path).unwrap().is_some());

        cache.remove_file(&file_path).unwrap();
        assert!(cache.get_file(&file_path).unwrap().is_none());
    }
}
