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

    // Build title to ID mapping for link resolution
    let title_to_id: HashMap<String, String> = notes.iter()
        .map(|(id, _, title, _)| (title.clone(), id.clone()))
        .collect();

    // Create nodes
    for (id, _path, title, folder) in &notes {
        nodes.push(GraphNode {
            id: id.clone(),
            title: title.clone(),
            folder: folder.clone()
        });
    }

    // Clear existing links
    conn.execute("DELETE FROM links", [])?;

    // Rebuild links from note content
    for (id, path, _title, _folder) in &notes {
        // Read note content from markdown file
        let note_path = notes_dir.join(path);
        if let Ok(content) = std::fs::read_to_string(&note_path) {
            // Extract links from content
            let links = LinkParser::extract_links(&content);

            for link_target in links {
                // Try to resolve link to note ID
                if let Some(target_id) = title_to_id.get(&link_target) {
                    // Add edge to graph
                    edges.push(GraphEdge {
                        source: id.clone(),
                        target: target_id.clone(),
                    });

                    // Store link in database
                    conn.execute(
                        "INSERT OR REPLACE INTO links (source_note_id, target_note_id, context) VALUES (?1, ?2, ?3)",
                        params![id, target_id, format!("[[{}]]", link_target)]
                    )?;
                }
            }
        }
    }

    Ok(GraphData { nodes, edges })
}
