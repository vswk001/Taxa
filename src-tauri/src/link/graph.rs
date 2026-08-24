// src-tauri/src/link/graph.rs
use crate::error::AppResult;
use crate::link::parser::LinkParser;
use crate::storage::database::Database;
use rusqlite::params;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

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

/// Title -> note id map for wikilink resolution. Ordered by created_at so a
/// duplicate title deterministically resolves to the oldest note.
fn title_map(conn: &rusqlite::Connection) -> AppResult<HashMap<String, String>> {
    let mut stmt = conn.prepare("SELECT id, title FROM notes ORDER BY created_at ASC, id ASC")?;
    let mut map: HashMap<String, String> = HashMap::new();
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    for r in rows {
        let (id, title) = r?;
        map.entry(title).or_insert(id);
    }
    Ok(map)
}

/// Recompute the outgoing links of one note from its content. Called from the
/// note create/update paths so the links table stays fresh without a full
/// vault rebuild on every graph view.
pub fn update_note_links(db: &Database, note_id: &str, content: &str) -> AppResult<()> {
    let links = LinkParser::extract_links(content);
    let conn = db.conn();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "DELETE FROM links WHERE source_note_id = ?1",
        params![note_id],
    )?;
    if !links.is_empty() {
        let titles = title_map(&tx)?;
        for target_title in links {
            if let Some(target_id) = titles.get(&target_title) {
                if target_id != note_id {
                    tx.execute(
                        "INSERT OR REPLACE INTO links (source_note_id, target_note_id, context) VALUES (?1, ?2, ?3)",
                        params![note_id, target_id, format!("[[{}]]", target_title)],
                    )?;
                }
            }
        }
    }
    tx.commit()?;
    Ok(())
}

/// Full rebuild from note files: used once at startup to reconcile legacy
/// link rows (the links table used to be rebuilt on every graph view).
pub fn rebuild_links(db: &Database, notes_dir: &Path) -> AppResult<()> {
    let conn = db.conn();
    let titles = title_map(conn)?;

    let mut stmt = conn.prepare("SELECT id, path FROM notes ORDER BY created_at ASC, id ASC")?;
    let notes: Vec<(String, String)> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    drop(stmt);

    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM links", [])?;
    for (id, path) in &notes {
        let full = notes_dir.join(path);
        let content = match std::fs::read_to_string(&full) {
            Ok(c) => c,
            Err(_) => continue, // phantom note; skip quietly
        };
        for target_title in LinkParser::extract_links(&content) {
            if let Some(target_id) = titles.get(&target_title) {
                if target_id != id {
                    tx.execute(
                        "INSERT OR REPLACE INTO links (source_note_id, target_note_id, context) VALUES (?1, ?2, ?3)",
                        params![id, target_id, format!("[[{}]]", target_title)],
                    )?;
                }
            }
        }
    }
    tx.commit()?;
    Ok(())
}

/// Graph data for the view: a plain SELECT over notes + links — no file
/// reads, no table writes.
pub fn get_graph_data(db: &Database, notes_dir: &Path) -> AppResult<GraphData> {
    let conn = db.conn();

    let mut stmt =
        conn.prepare("SELECT id, path, title, folder FROM notes ORDER BY created_at ASC, id ASC")?;
    let all: Vec<(String, String, String, String)> = stmt
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    drop(stmt);

    // Filter phantom notes (in DB but file missing on disk)
    let valid: Vec<(String, String, String)> = all
        .into_iter()
        .filter(|(_, path, _, _)| notes_dir.join(path).exists())
        .map(|(id, _, title, folder)| (id, title, folder))
        .collect();
    let valid_ids: std::collections::HashSet<&String> = valid.iter().map(|(id, _, _)| id).collect();

    let nodes = valid
        .iter()
        .map(|(id, title, folder)| GraphNode {
            id: id.clone(),
            title: title.clone(),
            folder: folder.clone(),
        })
        .collect();

    let mut stmt = conn.prepare("SELECT source_note_id, target_note_id FROM links")?;
    let mut edges = Vec::new();
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    for r in rows {
        let (source, target) = r?;
        if valid_ids.contains(&source) && valid_ids.contains(&target) {
            edges.push(GraphEdge { source, target });
        }
    }

    Ok(GraphData { nodes, edges })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::markdown::MarkdownStorage;

    fn test_env() -> (Database, MarkdownStorage, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("taxa-graph-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = Database::new(&dir.join("test.db")).unwrap();
        let md = MarkdownStorage::new(dir.join("notes"));
        std::fs::create_dir_all(dir.join("notes")).unwrap();
        (db, md, dir)
    }

    #[test]
    fn links_update_incrementally() {
        let (db, md, dir) = test_env();
        let a = crate::notebook::service::NotebookService::create_note(
            &db,
            &md,
            crate::notebook::model::CreateNoteRequest {
                folder: "".into(),
                title: "A".into(),
                content: "link to [[B]]".into(),
                tags: None,
            },
        )
        .unwrap();
        // B is created after A, so A's original link to [[B]] could not
        // resolve; touching A's content recomputes its outgoing links.
        let b = crate::notebook::service::NotebookService::create_note(
            &db,
            &md,
            crate::notebook::model::CreateNoteRequest {
                folder: "".into(),
                title: "B".into(),
                content: "back to [[A]] and [[A]] again".into(),
                tags: None,
            },
        )
        .unwrap();
        assert_eq!(
            get_graph_data(&db, &dir.join("notes")).unwrap().edges.len(),
            1
        ); // B->A only

        crate::notebook::service::NotebookService::update_note(
            &db,
            &md,
            crate::notebook::model::UpdateNoteRequest {
                id: a.id.clone(),
                title: None,
                content: Some("link to [[B]]".into()),
                folder: None,
                tags: None,
            },
        )
        .unwrap();
        let data = get_graph_data(&db, &dir.join("notes")).unwrap();
        assert_eq!(data.nodes.len(), 2);
        assert_eq!(data.edges.len(), 2); // A->B and B->A (dup collapsed)

        // Update A to remove its link; edges shrink without a rebuild.
        crate::notebook::service::NotebookService::update_note(
            &db,
            &md,
            crate::notebook::model::UpdateNoteRequest {
                id: a.id.clone(),
                title: None,
                content: Some("no links now".into()),
                folder: None,
                tags: None,
            },
        )
        .unwrap();
        let data = get_graph_data(&db, &dir.join("notes")).unwrap();
        assert_eq!(data.edges.len(), 1);
        assert_eq!(data.edges[0].source, b.id);
        let _ = b;
    }
}
