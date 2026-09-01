// src-tauri/src/commands/backup.rs
// Vault backup/restore. Backup zips a consistent database snapshot
// (VACUUM INTO, so WAL state is folded in) plus the notebooks and trash
// trees. Restore extracts to a staging dir and completes on the next
// launch via a marker file — replacing live database files under a
// running process would corrupt them.
use crate::error::{AppError, AppResult};
use crate::state::{lock_db, AppState};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::{DialogExt, FilePath};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

const RESTORE_MARKER: &str = "pending-restore.json";
const STAGING_DIR: &str = "restore-staging";

async fn pick_save_zip(app: &AppHandle, title: &str) -> AppResult<Option<PathBuf>> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_title(title)
        .add_filter("Zip", &["zip"])
        .set_file_name("taxa-backup.zip")
        .save_file(move |path| {
            let _ = tx.send(path);
        });
    rx.await
        .map_err(|_| AppError::Config("Dialog failed".into()))?
        .map(|p| match p {
            FilePath::Path(p) => Ok(p),
            FilePath::Url(u) => u
                .to_file_path()
                .map_err(|_| AppError::Config("Invalid file URL".into())),
        })
        .transpose()
}

async fn pick_zip(app: &AppHandle, title: &str) -> AppResult<Option<PathBuf>> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_title(title)
        .add_filter("Zip", &["zip"])
        .pick_file(move |path| {
            let _ = tx.send(path);
        });
    rx.await
        .map_err(|_| AppError::Config("Dialog failed".into()))?
        .map(|p| match p {
            FilePath::Path(p) => Ok(p),
            FilePath::Url(u) => u
                .to_file_path()
                .map_err(|_| AppError::Config("Invalid file URL".into())),
        })
        .transpose()
}

fn add_dir_to_zip(
    zip: &mut ZipWriter<File>,
    base: &Path,
    dir: &Path,
    prefix: &str,
) -> AppResult<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in walkdir(dir)? {
        if !entry.is_file() {
            continue;
        }
        let rel = entry
            .strip_prefix(base)
            .map_err(|e| AppError::FileIo(e.to_string()))?;
        let name = format!("{prefix}/{}", rel.to_string_lossy().replace('\\', "/"));
        zip.start_file(name, SimpleFileOptions::default())
            .map_err(zip_err)?;
        std::fs::File::open(&entry)
            .and_then(|mut f| {
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)?;
                zip.write_all(&buf)
            })
            .map_err(|e| AppError::FileIo(e.to_string()))?;
    }
    Ok(())
}

/// Iterates files under `dir` without following symlink cycles.
fn walkdir(dir: &Path) -> AppResult<Vec<PathBuf>> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in std::fs::read_dir(&current)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && !path.is_symlink() {
                stack.push(path);
            } else if path.is_file() {
                out.push(path);
            }
        }
    }
    Ok(out)
}

fn zip_err(e: zip::result::ZipError) -> AppError {
    AppError::FileIo(e.to_string())
}

/// Zip a consistent snapshot: DB via VACUUM INTO + notebooks/ + trash/.
#[tauri::command]
pub async fn backup_vault(app: AppHandle, state: State<'_, Arc<AppState>>) -> AppResult<bool> {
    let zip_path = match pick_save_zip(&app, "备份数据").await? {
        Some(p) => p,
        None => return Ok(false),
    };
    let state = state.inner().clone();
    crate::commands::notebook::run_blocking(move || -> AppResult<()> {
        // Consistent DB snapshot that folds in any WAL state.
        let snapshot = state.data_dir.join("backup-snapshot.db");
        let _ = std::fs::remove_file(&snapshot);
        {
            let db = lock_db(&state)?;
            db.conn().execute(
                "VACUUM INTO ?1",
                rusqlite::params![snapshot.to_string_lossy()],
            )?;
        }

        let file = std::fs::File::create(&zip_path)?;
        let mut zip = ZipWriter::new(file);
        zip.start_file("taxa.db", SimpleFileOptions::default())
            .map_err(zip_err)?;
        std::fs::File::open(&snapshot)
            .and_then(|mut f| {
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)?;
                zip.write_all(&buf)
            })
            .map_err(|e| AppError::FileIo(e.to_string()))?;

        // The whole notebooks tree (notes + attachments) plus trash.
        let notebooks = state.data_dir.join("notebooks");
        add_dir_to_zip(&mut zip, &state.data_dir, &notebooks, "notebooks")?;
        add_dir_to_zip(&mut zip, &state.data_dir, &state.trash_dir(), "trash")?;

        zip.finish().map_err(zip_err)?;
        let _ = std::fs::remove_file(&snapshot);
        Ok(())
    })
    .await?;
    Ok(true)
}

/// Stage a restore: extract the zip, leave a marker, and let the caller
/// relaunch. The actual swap happens in lib.rs setup on the next start.
#[tauri::command]
pub async fn restore_vault(app: AppHandle, state: State<'_, Arc<AppState>>) -> AppResult<bool> {
    let zip_path = match pick_zip(&app, "选择备份文件").await? {
        Some(p) => p,
        None => return Ok(false),
    };
    let state = state.inner().clone();
    crate::commands::notebook::run_blocking(move || -> AppResult<()> {
        let staging = state.data_dir.join(STAGING_DIR);
        if staging.exists() {
            std::fs::remove_dir_all(&staging)?;
        }
        std::fs::create_dir_all(&staging)?;

        let file = std::fs::File::open(&zip_path)?;
        let mut archive = zip::ZipArchive::new(file).map_err(zip_err)?;
        archive
            .extract(&staging)
            .map_err(|e| AppError::FileIo(format!("invalid backup archive: {e}")))?;

        if !staging.join("taxa.db").exists() {
            let _ = std::fs::remove_dir_all(&staging);
            return Err(AppError::Config("备份文件中缺少 taxa.db".into()));
        }

        std::fs::write(
            state.data_dir.join(RESTORE_MARKER),
            serde_json::json!({ "staging": STAGING_DIR }).to_string(),
        )?;
        Ok(())
    })
    .await?;
    Ok(true)
}

/// Runs in lib.rs setup BEFORE the database is opened: swap staged data in.
pub fn complete_pending_restore(data_dir: &Path) {
    let marker = data_dir.join(RESTORE_MARKER);
    let staging = data_dir.join(STAGING_DIR);
    if !marker.exists() {
        return;
    }
    eprintln!("[restore] pending restore found, applying staged data");
    let applied = (|| -> AppResult<()> {
        // Remove the live data; keep anything the backup doesn't cover out.
        for name in ["taxa.db", "taxa.db-wal", "taxa.db-shm", "taxis.db"] {
            let _ = std::fs::remove_file(data_dir.join(name));
        }
        let _ = std::fs::remove_dir_all(data_dir.join("notebooks"));
        let _ = std::fs::remove_dir_all(data_dir.join("trash"));

        for entry in std::fs::read_dir(&staging)? {
            let entry = entry?;
            let dest = data_dir.join(entry.file_name());
            if entry.path().is_dir() {
                if std::fs::rename(entry.path(), &dest).is_err() {
                    copy_dir_recursive(&entry.path(), &dest)?;
                }
            } else {
                std::fs::rename(entry.path(), &dest)?;
            }
        }
        Ok(())
    })();
    match applied {
        Ok(()) => eprintln!("[restore] applied successfully"),
        Err(e) => eprintln!("[restore] FAILED: {e} (old data files were removed)"),
    }
    let _ = std::fs::remove_dir_all(&staging);
    let _ = std::fs::remove_file(&marker);
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> AppResult<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let target = dest.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
