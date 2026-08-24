// mcp/src/resources.rs
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
/// Errors as `(jsonrpc_code, message)`: -32602 for malformed requests,
/// -32603 for lookup/IO failures.
pub fn read(params: &Value, ctx: &Ctx) -> Result<Value, (i64, String)> {
    let uri = params
        .get("uri")
        .and_then(|v| v.as_str())
        .ok_or((-32602, "missing 'uri'".to_string()))?;
    let id = uri.strip_prefix("taxa://notes/").ok_or_else(|| {
        (
            -32602,
            format!("unsupported URI (use taxa://notes/{{id}}): {}", uri),
        )
    })?;

    let (_note, content) =
        NotebookService::get_note(&ctx.db, &ctx.md, id).map_err(|e| (-32603, e.to_string()))?;
    Ok(json!({
        "contents": [{
            "uri": uri,
            "mimeType": "text/markdown",
            "text": content
        }]
    }))
}
