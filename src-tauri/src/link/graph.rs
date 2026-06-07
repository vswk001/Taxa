// src-tauri/src/link/graph.rs
use crate::error::AppResult;
use crate::link::parser::LinkParser;
use crate::storage::database::Database;
use serde::Serialize;
use std::collections::HashMap;
use rusqlite::params;

#[derive(Debug, Clone, Serialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphNode {
    pub id: String,
    pub title: String,
    pub folder: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
}

pub fn build_graph(db: &Database, notes_dir: &std::path::Path) -> AppResult<GraphData> {
    let conn = db.conn();

    // Get all notes from database
    let mut stmt = conn.prepare("SELECT id, path, title, folder FROM notes")?;
    let notes: Vec<(String, String, String, String)> = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    })?.collect::<Result<Vec<_>, _>>()?;

    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    // Filter out phantom notes (in DB but file missing on disk)
    let valid_notes: Vec<_> = notes.into_iter()
        .filter(|(_, path, _, _)| notes_dir.join(path).exists())
        .collect();

    // Build title to ID mapping for link resolution
    let title_to_id: HashMap<String, String> = valid_notes.iter()
        .map(|(id, _, title, _)| (title.clone(), id.clone()))
        .collect();

    // Create nodes
    for (id, _path, title, folder) in &valid_notes {
        nodes.push(GraphNode {
            id: id.clone(),
            title: title.clone(),
            folder: folder.clone()
        });
    }

    // Clear existing links
    conn.execute("DELETE FROM links", [])?;

    // Rebuild links from note content
    for (id, path, _title, _folder) in &valid_notes {
        let note_path = notes_dir.join(path);
        match std::fs::read_to_string(&note_path) {
            Ok(content) => {
                let links = LinkParser::extract_links(&content);
                if !links.is_empty() {
                    eprintln!("[GRAPH] note '{}' ({}) has {} links: {:?}", _title, path, links.len(), links);
                }
                for link_target in links {
                    if let Some(target_id) = title_to_id.get(&link_target) {
                        eprintln!("[GRAPH] resolved link '{}' -> note {}", link_target, target_id);
                        edges.push(GraphEdge {
                            source: id.clone(),
                            target: target_id.clone(),
                        });
                        conn.execute(
                            "INSERT OR REPLACE INTO links (source_note_id, target_note_id, context) VALUES (?1, ?2, ?3)",
                            params![id, target_id, format!("[[{}]]", link_target)]
                        )?;
                    } else {
                        eprintln!("[GRAPH] link '{}' not resolved (no matching title)", link_target);
                    }
                }
            }
            Err(e) => {
                eprintln!("[GRAPH] failed to read note '{}': {} (path: {:?})", _title, e, note_path);
            }
        }
    }
    eprintln!("[GRAPH] total: {} nodes, {} edges", nodes.len(), edges.len());

    Ok(GraphData { nodes, edges })
}
