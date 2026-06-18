// src-tauri/src/bin/mcp/tools.rs
// Tool metadata + handlers. All operations are read-only.
use crate::Ctx;
use crate::snippet;
use serde_json::{json, Value};
use std::collections::HashMap;
use taxis_lib::link::parser::LinkParser;
use taxis_lib::notebook::service::NotebookService;
use taxis_lib::storage::markdown::MarkdownStorage;

/// JSON Schema descriptors advertised via `tools/list`.
pub fn list() -> Vec<Value> {
    vec![
        json!({
            "name": "search_notes",
            "description": "Search the Taxis knowledge base by keyword. Returns matching notes (id, title, folder, path, updated_at, snippet). Use this first, then read_note for full content.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search term (matches title, tags, and content)." },
                    "scope": { "type": "string", "enum": ["all", "title", "tags", "content"], "default": "all" },
                    "limit": { "type": "integer", "default": 20, "maximum": 50 }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": "read_note",
            "description": "Read a single note's full content and metadata by id.",
            "inputSchema": {
                "type": "object",
                "properties": { "id": { "type": "string" } },
                "required": ["id"]
            }
        }),
        json!({
            "name": "list_notes",
            "description": "List notes in a folder, or the most recently updated notes across all folders when no folder is given.",
            "inputSchema": {
                "type": "object",
                "properties": { "folder": { "type": "string", "description": "Folder path; omit for recent notes." } }
            }
        }),
        json!({
            "name": "get_folder_tree",
            "description": "Return the folder structure of the notebook with note counts per folder.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "get_backlinks",
            "description": "Find notes that link to a given note via [[wikilink]] (by exact title). Useful for discovering related notes.",
            "inputSchema": {
                "type": "object",
                "properties": { "title": { "type": "string", "description": "Exact title of the target note." } },
                "required": ["title"]
            }
        }),
    ]
}

/// Dispatch a tools/call. Always returns a CallToolResult; logical failures are
/// reported with `isError: true` rather than as JSON-RPC errors.
pub fn call(params: &Value, ctx: &Ctx) -> Value {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let args = params.get("arguments").cloned().unwrap_or(json!({}));
    let (text, is_error) = match dispatch(name, &args, ctx) {
        Ok(t) => (t, false),
        Err(e) => (e, true),
    };
    json!({
        "content": [{ "type": "text", "text": text }],
        "isError": is_error
    })
}

fn dispatch(name: &str, args: &Value, ctx: &Ctx) -> Result<String, String> {
    match name {
        "search_notes" => search_notes(args, ctx),
        "read_note" => read_note(args, ctx),
        "list_notes" => list_notes(args, ctx),
        "get_folder_tree" => folder_tree(ctx),
        "get_backlinks" => backlinks(args, ctx),
        other => Err(format!("Unknown tool: {}", other)),
    }
}

fn search_notes(args: &Value, ctx: &Ctx) -> Result<String, String> {
    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or("missing required argument 'query'")?;

    // Empty query: degrade to recent notes (still useful to the AI).
    if query.trim().is_empty() {
        let notes = NotebookService::list_recent_notes(&ctx.db, 20).map_err(|e| e.to_string())?;
        return Ok(pretty(&json!({ "results": note_summaries(&notes), "count": notes.len() })));
    }

    let scope = args.get("scope").and_then(|v| v.as_str()).unwrap_or("all");
    let limit = args
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(20)
        .min(50) as usize;

    let results = NotebookService::search_notes(&ctx.db, query, scope).map_err(|e| e.to_string())?;
    let ids: Vec<&str> = results.iter().map(|r| r.id.as_str()).collect();
    let meta = query_meta(ctx, &ids)?; // id -> (folder, updated_at)

    let mut out: Vec<Value> = Vec::new();
    for r in results.iter().take(limit) {
        let (folder, updated) = meta.get(r.id.as_str()).cloned().unwrap_or_default();
        // Best-effort keyword snippet from content (notes are typically small).
        let snippet = MarkdownStorage::read_note(&ctx.md, &r.path)
            .ok()
            .and_then(|content| snippet::extract_window(&content, query, 80));
        out.push(json!({
            "id": r.id,
            "title": r.title,
            "folder": folder,
            "path": r.path,
            "updated_at": updated,
            "snippet": snippet,
        }));
    }
    Ok(pretty(&json!({ "results": out, "count": out.len() })))
}

fn read_note(args: &Value, ctx: &Ctx) -> Result<String, String> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or("missing required argument 'id'")?;
    let (note, content) = NotebookService::get_note(&ctx.db, &ctx.md, id).map_err(|e| e.to_string())?;
    let v = json!({
        "id": note.id,
        "title": note.title,
        "folder": note.folder,
        "tags": note.tags,
        "created_at": note.created_at,
        "updated_at": note.updated_at,
        "word_count": note.word_count,
        "summary": note.summary,
        "content": content,
    });
    Ok(pretty(&v))
}

fn list_notes(args: &Value, ctx: &Ctx) -> Result<String, String> {
    let folder = args.get("folder").and_then(|v| v.as_str());
    let notes = match folder {
        Some(f) => NotebookService::list_notes(&ctx.db, &ctx.md, f).map_err(|e| e.to_string())?,
        None => NotebookService::list_recent_notes(&ctx.db, 50).map_err(|e| e.to_string())?,
    };
    Ok(pretty(&json!({ "notes": note_summaries(&notes), "count": notes.len() })))
}

fn folder_tree(ctx: &Ctx) -> Result<String, String> {
    let tree = NotebookService::get_folder_tree(&ctx.md).map_err(|e| e.to_string())?;
    Ok(pretty(&json!({ "folders": tree })))
}

/// Pure read: scan note content for `[[title]]` wikilinks. No DB writes
/// (unlike build_graph, which mutates the links table).
fn backlinks(args: &Value, ctx: &Ctx) -> Result<String, String> {
    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or("missing required argument 'title'")?;

    let conn = ctx.db.conn();
    let mut stmt = conn
        .prepare("SELECT id, path, title, folder FROM notes")
        .map_err(|e| e.to_string())?;
    let rows: Vec<(String, String, String, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut out: Vec<Value> = Vec::new();
    for (id, path, note_title, folder) in rows {
        if let Ok(content) = ctx.md.read_note(&path) {
            let links = LinkParser::extract_links(&content);
            if links.iter().any(|l| l == title) {
                out.push(json!({ "id": id, "title": note_title, "folder": folder }));
            }
        }
    }
    Ok(pretty(&json!({ "backlinks": out, "count": out.len() })))
}

// ---- helpers ----

fn note_summaries(notes: &[taxis_lib::notebook::model::Note]) -> Vec<Value> {
    notes
        .iter()
        .map(|n| {
            json!({
                "id": n.id,
                "title": n.title,
                "folder": n.folder,
                "updated_at": n.updated_at,
                "summary": n.summary,
            })
        })
        .collect()
}

/// Batched lookup of (folder, updated_at) by note id — one query, no per-row reads.
fn query_meta(ctx: &Ctx, ids: &[&str]) -> Result<HashMap<String, (String, String)>, String> {
    let mut map = HashMap::new();
    if ids.is_empty() {
        return Ok(map);
    }
    let placeholders = (0..ids.len()).map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT id, folder, updated_at FROM notes WHERE id IN ({})",
        placeholders
    );
    let conn = ctx.db.conn();
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let params: Vec<&dyn rusqlite::types::ToSql> =
        ids.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();
    let rows = stmt
        .query_map(params.as_slice(), |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
        })
        .map_err(|e| e.to_string())?;
    for row in rows {
        let (id, folder, updated) = row.map_err(|e| e.to_string())?;
        map.insert(id, (folder, updated));
    }
    Ok(map)
}

fn pretty(v: &Value) -> String {
    serde_json::to_string_pretty(v).unwrap_or_else(|_| "{}".into())
}
