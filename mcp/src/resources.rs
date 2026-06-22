// src-tauri/src/bin/mcp/resources.rs
// Expose notes as addressable resources under taxa://notes/{id}.
use crate::Ctx;
use serde_json::{json, Value};
use taxa_lib::notebook::service::NotebookService;

/// List recent notes as resources (capped) so clients can discover/autocomplete.
pub fn list(ctx: &Ctx) -> Value {
    let notes = NotebookService::list_recent_notes(&ctx.db, 50).unwrap_or_default();
    let resources: Vec<Value> = notes
        .iter()
        .map(|n| {
            json!({
                "uri": format!("taxa://notes/{}", n.id),
                "name": n.title,
                "mimeType": "text/markdown",
                "description": n.summary,
            })
        })
        .collect();
    json!({ "resources": resources })
}

/// Read a resource by URI. Only `taxa://notes/{id}` is supported in v1.
pub fn read(params: &Value, ctx: &Ctx) -> Result<Value, String> {
    let uri = params
        .get("uri")
        .and_then(|v| v.as_str())
        .ok_or("missing 'uri'")?;
    let id = uri
        .strip_prefix("taxa://notes/")
        .ok_or_else(|| format!("unsupported URI (use taxa://notes/{{id}}): {}", uri))?;

    let (_note, content) =
        NotebookService::get_note(&ctx.db, &ctx.md, id).map_err(|e| e.to_string())?;
    Ok(json!({
        "contents": [{
            "uri": uri,
            "mimeType": "text/markdown",
            "text": content
        }]
    }))
}
