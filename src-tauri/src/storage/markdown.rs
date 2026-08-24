// src-tauri/src/storage/markdown.rs
use crate::error::{AppError, AppResult};
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

pub struct MarkdownStorage {
    base_dir: PathBuf,
}

impl MarkdownStorage {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Validate a vault-relative folder path. Must be relative and free of
    /// `..`/`.`/absolute components and Windows reserved device names, so that
    /// caller-supplied (or LLM-supplied) strings can never escape the vault.
    pub fn validate_folder(folder: &str) -> AppResult<()> {
        if folder.is_empty() {
            return Ok(());
        }
        let path = Path::new(folder);
        if path.is_absolute() {
            return Err(AppError::InvalidPath(folder.to_string()));
        }
        for comp in path.components() {
            match comp {
                Component::Normal(part) => {
                    let s = part.to_string_lossy();
                    // ':' only occurs in Windows drive prefixes ("C:/x") and
                    // reserved stream names — reject it on every platform so
                    // validation behaves identically everywhere (on Unix,
                    // "C:/abs" is otherwise just a folder literally named "C:").
                    if s.is_empty() || s.contains(':') || is_windows_reserved(&s) {
                        return Err(AppError::InvalidPath(folder.to_string()));
                    }
                }
                // CurDir, ParentDir, RootDir, Prefix — none are allowed.
                _ => return Err(AppError::InvalidPath(folder.to_string())),
            }
        }
        Ok(())
    }

    pub fn base(&self) -> &Path {
        &self.base_dir
    }

    /// Resolve a folder to an absolute path inside the vault.
    fn dir(&self, folder: &str) -> AppResult<PathBuf> {
        Self::validate_folder(folder)?;
        Ok(if folder.is_empty() {
            self.base_dir.clone()
        } else {
            self.base_dir.join(folder)
        })
    }

    /// Resolve a DB-stored relative file path, enforcing containment.
    fn file(&self, relative_path: &str) -> AppResult<PathBuf> {
        let path = self.base_dir.join(relative_path);
        if !path.starts_with(&self.base_dir) {
            return Err(AppError::InvalidPath(relative_path.to_string()));
        }
        Ok(path)
    }

    /// Vault-relative path, always with `/` separators regardless of platform,
    /// so DB-stored paths match folder-path prefix logic everywhere.
    fn rel(&self, path: &Path) -> AppResult<String> {
        path.strip_prefix(&self.base_dir)
            .map_err(|_| AppError::FileIo("Failed to compute relative path".into()))
            .map(|p| p.to_string_lossy().replace('\\', "/"))
    }

    pub fn create_note(&self, folder: &str, filename: &str, content: &str) -> AppResult<PathBuf> {
        let dir = self.dir(folder)?;
        fs::create_dir_all(&dir)?;
        let sanitized = sanitize_filename(filename);
        // create_new avoids the exists()/write() TOCTOU race: the OS rejects
        // concurrent creation of the same name, so auto-suffixing is safe.
        let mut i = 0;
        loop {
            let name = if i == 0 {
                format!("{}.md", sanitized)
            } else {
                format!("{}-{}.md", sanitized, i)
            };
            let path = dir.join(&name);
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(mut file) => {
                    file.write_all(content.as_bytes())?;
                    return Ok(path);
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    i += 1;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }

    pub fn read_note(&self, relative_path: &str) -> AppResult<String> {
        let path = self.file(relative_path)?;
        let content = fs::read_to_string(&path)?;
        Ok(content)
    }

    pub fn update_note(&self, relative_path: &str, content: &str) -> AppResult<()> {
        let path = self.file(relative_path)?;
        if !path.exists() {
            return Err(AppError::NotFound(format!(
                "Note not found: {}",
                relative_path
            )));
        }
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn delete_note(&self, relative_path: &str) -> AppResult<()> {
        let path = self.file(relative_path)?;
        if !path.exists() {
            return Err(AppError::NotFound(format!(
                "Note not found: {}",
                relative_path
            )));
        }
        fs::remove_file(&path)?;
        Ok(())
    }

    /// Move/rename a note file. Never overwrites: a destination that already
    /// exists (and is not the source itself) gets an auto-suffix, matching
    /// create_note behavior, so a title collision cannot destroy another note.
    pub fn move_note(&self, from: &str, to_folder: &str, to_filename: &str) -> AppResult<String> {
        let src = self.file(from)?;
        if !src.exists() {
            return Err(AppError::NotFound(format!("Note not found: {}", from)));
        }
        let dest_dir = self.dir(to_folder)?;
        fs::create_dir_all(&dest_dir)?;
        let sanitized = sanitize_filename(to_filename);

        let mut dest = dest_dir.join(format!("{}.md", sanitized));
        let mut i = 1;
        while dest.exists() && !same_file(&src, &dest) {
            dest = dest_dir.join(format!("{}-{}.md", sanitized, i));
            i += 1;
        }
        if !same_file(&src, &dest) {
            fs::rename(&src, &dest)?;
        }
        self.rel(&dest)
    }

    pub fn list_folder(&self, folder: &str) -> AppResult<Vec<String>> {
        let dir = self.dir(folder)?;
        if !dir.exists() {
            return Ok(vec![]);
        }
        let mut entries = vec![];
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".md") {
                entries.push(name);
            }
        }
        entries.sort();
        Ok(entries)
    }

    pub fn list_subfolders(&self, folder: &str) -> AppResult<Vec<String>> {
        let dir = self.dir(folder)?;
        if !dir.exists() {
            return Ok(vec![]);
        }
        let mut entries = vec![];
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                entries.push(entry.file_name().to_string_lossy().to_string());
            }
        }
        entries.sort();
        Ok(entries)
    }

    pub fn full_path(&self, relative_path: &str) -> PathBuf {
        // Containment-checked for consistency with the other accessors.
        self.file(relative_path)
            .unwrap_or_else(|_| self.base_dir.join("invalid"))
    }

    pub fn create_folder(&self, parent: &str, name: &str) -> AppResult<String> {
        // Sanitizing first keeps "a/b" from becoming a nested path: the result
        // is always a single safe component under `parent`.
        let sanitized = sanitize_filename(name);
        Self::validate_folder(&sanitized)?;
        let path = self.dir(parent)?.join(&sanitized);
        fs::create_dir_all(&path)?;
        self.rel(&path)
    }

    pub fn rename_folder(&self, old_path: &str, new_name: &str) -> AppResult<String> {
        Self::validate_folder(old_path)?;
        let sanitized = sanitize_filename(new_name);
        let old = self.dir(old_path)?;
        if !old.exists() {
            return Err(AppError::NotFound(format!(
                "Folder not found: {}",
                old_path
            )));
        }
        let parent = old
            .parent()
            .ok_or_else(|| AppError::FileIo("Invalid path".into()))?;
        let new = parent.join(&sanitized);
        if new.exists() && !same_file(&old, &new) {
            return Err(AppError::FileIo(format!(
                "Folder already exists: {}",
                sanitized
            )));
        }
        if !same_file(&old, &new) {
            fs::rename(&old, &new)?;
        }
        self.rel(&new)
    }

    pub fn delete_folder(&self, path: &str) -> AppResult<()> {
        let dir = self.dir(path)?;
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }
        Ok(())
    }
}

/// Case-insensitive comparison for the same path, so a rename that only
/// changes letter case is treated as a no-op on Windows.
fn same_file(a: &Path, b: &Path) -> bool {
    if a == b {
        return true;
    }
    #[cfg(windows)]
    {
        a.to_string_lossy().to_lowercase() == b.to_string_lossy().to_lowercase()
    }
    #[cfg(not(windows))]
    {
        false
    }
}

fn is_windows_reserved(name: &str) -> bool {
    const RESERVED: &[&str] = &[
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    let upper = name.to_ascii_uppercase();
    RESERVED.contains(&upper.as_str())
}

/// Make a user/LLM-supplied string safe to use as a single path component.
pub fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                '_'
            } else {
                c
            }
        })
        .collect();
    let trimmed = cleaned.trim_matches(|c: char| c == '.' || c.is_whitespace());
    if trimmed.is_empty() {
        return "untitled".to_string();
    }
    if is_windows_reserved(trimmed) {
        return format!("_{}", trimmed);
    }
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn storage() -> (MarkdownStorage, PathBuf) {
        let dir = std::env::temp_dir().join(format!("taxa-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        (MarkdownStorage::new(dir.clone()), dir)
    }

    #[test]
    fn validate_folder_rejects_traversal() {
        assert!(MarkdownStorage::validate_folder("").is_ok());
        assert!(MarkdownStorage::validate_folder("a/b").is_ok());
        assert!(MarkdownStorage::validate_folder("..").is_err());
        assert!(MarkdownStorage::validate_folder("a/../b").is_err());
        assert!(MarkdownStorage::validate_folder("a/../../x").is_err());
        assert!(MarkdownStorage::validate_folder("/abs").is_err());
        assert!(MarkdownStorage::validate_folder("C:/abs").is_err());
        assert!(MarkdownStorage::validate_folder("CON").is_err());
        assert!(MarkdownStorage::validate_folder("a/..").is_err());
    }

    #[test]
    fn create_note_stays_in_vault() {
        let (md, dir) = storage();
        let err = md.create_note("..", "escape", "x").unwrap_err();
        assert!(matches!(err, AppError::InvalidPath(_)));
        assert!(!dir.parent().unwrap().join("escape.md").exists());
    }

    #[test]
    fn move_note_does_not_overwrite() {
        let (md, _dir) = storage();
        md.create_note("", "A", "content A").unwrap();
        md.create_note("", "B", "content B").unwrap();
        // Rename A to title B — must auto-suffix, not clobber B.
        let new_rel = md.move_note("A.md", "", "B").unwrap();
        assert_eq!(new_rel, "B-1.md");
        assert!(md.read_note("B.md").unwrap().contains("content B"));
        assert!(md.read_note(&new_rel).unwrap().contains("content A"));
    }

    #[test]
    fn move_note_same_title_is_noop() {
        let (md, _dir) = storage();
        md.create_note("", "A", "x").unwrap();
        let rel = md.move_note("A.md", "", "A").unwrap();
        assert_eq!(rel, "A.md");
    }

    #[test]
    fn sanitize_handles_reserved_and_edges() {
        assert_eq!(sanitize_filename("CON"), "_CON");
        assert_eq!(sanitize_filename("trailing... "), "trailing");
        assert_eq!(sanitize_filename("a/b\\c:d"), "a_b_c_d");
        assert_eq!(sanitize_filename("..."), "untitled");
        assert_eq!(sanitize_filename(""), "untitled");
    }

    #[test]
    fn rename_folder_sanitizes_new_name() {
        let (md, _dir) = storage();
        md.create_folder("", "sub").unwrap();
        // "../evil" is sanitized into a single safe component — the folder
        // never leaves the vault.
        let new_rel = md.rename_folder("sub", "../evil").unwrap();
        assert_eq!(new_rel, "_evil");
        assert!(md.base().join("_evil").exists());
        assert!(!md.base().join("sub").exists());
    }

    #[test]
    fn rename_folder_conflict_errors() {
        let (md, _dir) = storage();
        md.create_folder("", "a").unwrap();
        md.create_folder("", "b").unwrap();
        assert!(md.rename_folder("a", "b").is_err());
    }
}
