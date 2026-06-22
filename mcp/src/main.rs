// src-tauri/src/bin/mcp/main.rs
// Taxa MCP server (read-only knowledge base) — entry point + stdio loop.
//
// Speaks JSON-RPC 2.0 over stdin/stdout, one message per line. Logging goes
// to stderr only; stdout is reserved for the protocol. Shares the app's
// storage + notebook modules via taxa_lib; opens the live database without
// running migrations and never writes.
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use taxa_lib::storage::database::Database;
use taxa_lib::storage::markdown::MarkdownStorage;

mod resources;
mod server;
mod snippet;
mod tools;

/// Shared, read-only context handed to every handler.
pub struct Ctx {
    pub db: Database,
    pub md: MarkdownStorage,
}

/// Resolve the data dir exactly as the Tauri app does.
fn data_dir() -> Option<PathBuf> {
    Some(dirs::data_dir()?.join("Taxis"))
}

fn main() {
    let data_dir = match data_dir() {
        Some(d) => d,
        None => {
            eprintln!("[taxa-mcp] cannot determine data directory");
            std::process::exit(1);
        }
    };
    let db_path = data_dir.join("taxis.db");
    let notes_dir = data_dir.join("notebooks").join("default").join("notes");

    let db = match Database::open_existing(&db_path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("[taxa-mcp] failed to open database at {}: {}", db_path.display(), e);
            std::process::exit(1);
        }
    };
    let md = MarkdownStorage::new(notes_dir);
    let ctx = Ctx { db, md };

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut handle = stdin.lock();

    for line in (&mut handle).lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let req = match serde_json::from_str::<serde_json::Value>(trimmed) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[taxa-mcp] JSON parse error: {}", e);
                continue;
            }
        };
        if let Some(resp) = server::handle(&req, &ctx) {
            let mut s = serde_json::to_string(&resp).unwrap_or_else(|_| "{}".into());
            s.push('\n');
            let _ = out.write_all(s.as_bytes());
            let _ = out.flush();
        }
    }
}
