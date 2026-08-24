// mcp/src/server.rs
// JSON-RPC 2.0 dispatch for the MCP server.
use crate::Ctx;
use serde_json::{json, Value};

/// Handle one inbound message. Returns the response to write, or `None` for
/// notifications (which carry no `id` and expect no reply).
pub fn handle(req: &Value, ctx: &Ctx) -> Option<Value> {
    let id = req.get("id").cloned();
    let method = req.get("method").and_then(|v| v.as_str()).unwrap_or("");
    let params = req.get("params").cloned().unwrap_or(Value::Null);

    let result: Result<Value, Value> = match method {
        "initialize" => Ok(initialize(&params)),
        "notifications/initialized" => return None, // notification, no reply
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": super::tools::list() })),
        "tools/call" => Ok(super::tools::call(&params, ctx)),
        "resources/list" => Ok(super::resources::list(ctx)),
        "resources/read" => {
            super::resources::read(&params, ctx).map_err(|(code, msg)| rpc_error(code, &msg))
        }
        _ => Err(rpc_error(-32601, &format!("Method not found: {}", method))),
    };

    // Drop notifications (no id) even if they produced a result.
    let id = id?;
    Some(match result {
        Ok(r) => json!({ "jsonrpc": "2.0", "id": id, "result": r }),
        Err(err) => json!({ "jsonrpc": "2.0", "id": id, "error": err }),
    })
}

fn rpc_error(code: i64, message: &str) -> Value {
    json!({ "code": code, "message": message })
}

/// MCP protocol versions this server understands. Anything else falls back to
/// the baseline instead of blindly echoing an unknown version.
const SUPPORTED_PROTOCOL_VERSIONS: &[&str] = &["2024-11-05", "2025-03-26", "2025-06-18"];

fn initialize(params: &Value) -> Value {
    let requested = params
        .get("protocolVersion")
        .and_then(|v| v.as_str())
        .unwrap_or("2024-11-05");
    let negotiated = if SUPPORTED_PROTOCOL_VERSIONS.contains(&requested) {
        requested
    } else {
        "2024-11-05"
    };
    json!({
        "protocolVersion": negotiated,
        "capabilities": {
            "tools": { "listChanged": false },
            "resources": { "listChanged": false }
        },
        "serverInfo": {
            "name": "taxa",
            "version": env!("CARGO_PKG_VERSION")
        }
    })
}
