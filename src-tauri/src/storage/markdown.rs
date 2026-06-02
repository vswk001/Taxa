// src-tauri/src/storage/markdown.rs
use crate::error::{AppError, AppResult};
use std::fs;
use std::path::PathBuf;

pub struct MarkdownStorage {
    base_dir: PathBuf,
}

impl MarkdownStorage {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    pub fn create_note(&self, folder: &str, filename: &str, content: &str) -> AppResult<PathBuf> {
        let dir = self.base_dir.join(folder);
        fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.md", sanitize_filename(filename)));
        if path.exists() {
            return Err(AppError::FileIo(format!("Note already exists: {}", path.display())));
        }
        fs::write(&path, content)?;
        Ok(path)
    }

    pub fn read_note(&self, relative_path: &str) -> AppResult<String> {
        let path = self.base_dir.join(relative_path);
        let content = fs::read_to_string(&path)?;
        Ok(content)
    }

    pub fn update_note(&self, relative_path: &str, content: &str) -> AppResult<()> {
        let path = self.base_dir.join(relative_path);
        if !path.exists() {
            return Err(AppError::NotFound(format!("Note not found: {}", relative_path)));
        }
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn delete_note(&self, relative_path: &str) -> AppResult<()> {
        let path = self.base_dir.join(relative_path);
        if !path.exists() {
            return Err(AppError::NotFound(format!("Note not found: {}", relative_path)));
        }
        fs::remove_file(&path)?;
        Ok(())
    }

    pub fn move_note(&self, from: &str, to_folder: &str, to_filename: &str) -> AppResult<String> {
        let src = self.base_dir.join(from);
        if !src.exists() {
            return Err(AppError::NotFound(format!("Note not found: {}", from)));
        }
        let dest_dir = self.base_dir.join(to_folder);
        fs::create_dir_all(&dest_dir)?;
        let dest = dest_dir.join(format!("{}.md", sanitize_filename(to_filename)));
        fs::rename(&src, &dest)?;
        Ok(dest.strip_prefix(&self.base_dir)
            .map_err(|_| AppError::FileIo("Failed to compute relative path".into()))?
            .to_string_lossy()
            .to_string())
    }

    pub fn list_folder(&self, folder: &str) -> AppResult<Vec<String>> {
        let dir = self.base_dir.join(folder);
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
        let dir = self.base_dir.join(folder);
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
        self.base_dir.join(relative_path)
    }

    pub fn create_folder(&self, parent: &str, name: &str) -> AppResult<String> {
        let path = if parent.is_empty() {
            self.base_dir.join(name)
        } else {
            self.base_dir.join(parent).join(name)
        };
        fs::create_dir_all(&path)?;
        Ok(path.strip_prefix(&self.base_dir)
            .map_err(|_| AppError::FileIo("Failed to compute relative path".into()))?
            .to_string_lossy()
            .to_string())
    }

    pub fn rename_folder(&self, old_path: &str, new_name: &str) -> AppResult<String> {
        let old = self.base_dir.join(old_path);
        let parent = old.parent().ok_or_else(|| AppError::FileIo("Invalid path".into()))?;
        let new = parent.join(new_name);
        fs::rename(&old, &new)?;
        Ok(new.strip_prefix(&self.base_dir)
            .map_err(|_| AppError::FileIo("Failed to compute relative path".into()))?
            .to_string_lossy()
            .to_string())
    }

    pub fn delete_folder(&self, path: &str) -> AppResult<()> {
        let dir = self.base_dir.join(path);
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }
        Ok(())
    }
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c == '/' || c == '\\' || c == ':' || c == '*' || c == '?' || c == '"' || c == '<' || c == '>' || c == '|' { '_' } else { c })
        .collect()
}
