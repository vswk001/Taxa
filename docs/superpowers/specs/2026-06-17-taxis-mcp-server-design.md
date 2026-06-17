# Taxis MCP Server — Design Spec

## Goal

Taxis exposes its local note knowledge base over the **Model Context Protocol (MCP)** so that AI tools — Claude Code, Codex, and any other MCP-compatible client — can search and read the user's notes as context during their work.

**v1 is read-only.** The architecture reserves write + delete for a later version.

## Non-Goals (v1)

- Writing, updating, moving, or deleting notes via MCP. (Architecture must allow adding these without redesign — see "Write/Delete Reservation".)
- HTTP / SSE transport. Local single-user use is served by stdio.
- Authentication / authorization. Local stdio on the user's own machine; no network exposure.
- Multi-notebook support. v1 serves the single `default` notebook the app already uses.

## Architecture

A **separate Rust binary**, `taxis-mcp`, built as an additional `[[bin]]` target in the existing `src-tauri` Cargo package. It shares the app's storage and notebook modules directly — no code duplication, no separate service layer.

```
┌─────────────────┐      spawn (stdio JSON-RPC)      ┌──────────────────┐
│  Claude Code    │ ─────────────────────────────────▶│   taxis-mcp      │
│  / Codex / ...  │ ◀─────────────────────────────────│  (Rust binary)   │
└─────────────────┘         MCP responses             └────────┬─────────┘
                                                                │ reads
                                       ┌────────────────────────┴────┐
                                       ▼                             ▼
                              ┌─────────────────┐         ┌──────────────────┐
                              │  SQLite (RO)    │         │  Markdown files  │
                              │  taxis.db       │         │  notebooks/...   │
                              └─────────────────┘         └──────────────────┘
                                       ▲                             ▲
                                       │ also reads/writes (RW)      │
                              ┌────────┴─────────────────────────────┘
                              │   shared via existing modules
                       ┌──────┴──────────┐
                       │  Taxis app      │
                       │  (Tauri, RW)    │
                       └─────────────────┘
```

**Why a separate binary, not a `--mcp` flag on the Tauri app:**
- Clients spawn the server as a long-lived subprocess and expect fast startup. The Tauri app carries a webview and is not meant to run headless.
- A dedicated binary has no GUI dependencies and is exactly what an `mcpServers` config points at.
- It reuses `Database`, `MarkdownStorage`, `NotebookService` — the same code path the app uses, so behavior is identical.

**Why the same Cargo package (second `[[bin]]`):**
- Shares all modules (`storage`, `notebook`, `link`) via the library target without a workspace or path dependency.
- One `cargo build` produces both `taxis` (app) and `taxis-mcp`.

### Package layout

```
src-tauri/
  Cargo.toml          # add [[bin]] taxis-mcp → src/bin/mcp/main.rs
  src/
    bin/
      mcp/
        main.rs       # entry: parse args, open RO db, run stdio server loop
        server.rs     # JSON-RPC dispatch (initialize, tools/*, resources/*)
        tools.rs      # tool handler implementations
        resources.rs  # resource handler implementations
        snippet.rs    # search-snippet extraction helper
    storage/...       # existing (shared)
    notebook/...      # existing (shared)
    link/...          # existing (shared)
```

The MCP code lives under `src/bin/mcp/` so it is compiled only into the `taxis-mcp` binary, keeping it out of the app's library.

## Transport: stdio

Standard for local MCP servers. The client (Claude Code) launches `taxis-mcp` as a subprocess and exchanges newline-delimited JSON-RPC 2.0 over stdin/stdout. Logging goes to **stderr** only (stdout is reserved for the protocol).

Implemented protocol methods:
- `initialize` — advertise capabilities `{ tools: {}, resources: {} }`, return server name/version and protocol version.
- `tools/list`, `tools/call`
- `resources/list`, `resources/read`
- `notifications/initialized` (ack, no response)

Target the current MCP spec version; pin the `protocolVersion` string the SDK reports.

## Data Access & Concurrency

`taxis-mcp` locates the data dir the **same way the app does** — `dirs::data_dir().join("Taxis")` — so DB and markdown paths resolve identically:
- SQLite: `{data_dir}/Taxis/taxis.db`
- Markdown: `{data_dir}/Taxis/notebooks/default/notes/`

It opens the SQLite connection **read-only** (`OpenOptions::new().read_only(true)`):
- The DB already runs in WAL mode (`PRAGMA journal_mode=WAL`, set in `run_migrations`). WAL permits many concurrent readers alongside one writer — a read-only connection **never contends** with the running Taxis app.
- Read-only is a hard guarantee: the MCP process physically cannot mutate notes, even before write tools exist.

A small `busy_timeout` is set defensively (harmless for RO, future-proofs a later RW connection).

## Capabilities (read-only v1)

### Tools

| Tool | Params | Returns | Wraps |
|------|--------|---------|-------|
| `search_notes` | `query` (req), `scope` (all\|title\|tags\|content, def all), `limit` (def 20, max 50) | `[{id, title, folder, path, updated_at, snippet}]` | `NotebookService::search_notes` + snippet |
| `read_note` | `id` (req) | `{id, title, folder, tags[], created_at, updated_at, word_count, summary, content}` | `get_note` |
| `list_notes` | `folder` (opt) | `[{id, title, folder, updated_at, summary}]` (metadata only, no content) | `list_notes` if folder given else `list_recent_notes` |
| `get_folder_tree` | — | nested `[{name, path, note_count, children[]}]` | `get_folder_tree` |
| `get_backlinks` | `title` (req) | `[{id, title, folder}]` notes that `[[link]]` to it | graph/link logic |

The high-value pair is **`search_notes` → `read_note`**: the AI finds relevant notes, then reads full content. `get_folder_tree` aids discovery; `get_backlinks` aids traversal across the wikilink graph.

**Return shapes** use the existing model structs (`Note`, `SearchResult`, `Folder`) serialized via their existing `Serialize` impls. `content` is omitted from list/search results to keep payloads small; `read_note` is the way to get full text.

### Resources

- `taxis://notes/{id}` → the note's markdown content (text/markdown).
- `taxis://folders/{path}` (optional, v1.1) → folder listing. Defer if low value.

Resources make notes addressable so a client can embed/reference a specific note.

## Search Snippet Enhancement

`NotebookService::search_notes` currently returns an **empty `snippet`** and static ranking. For MCP, a snippet (matched keyword ± N chars) materially helps the AI decide whether to fetch full content.

**Scope of change:** implement snippet extraction **inside the MCP binary** (`snippet.rs`), operating on the content returned by `get_note`-style reads of matched notes — **not** by modifying the app's `NotebookService`. This keeps the app's behavior unchanged and contains the enhancement to the MCP surface. If a future app feature also wants snippets, the helper can be promoted to a shared module.

## Error Handling

- DB or notes dir missing / unreadable → fail `initialize` or return a clear MCP error on first tool call with a human message ("Taxis data directory not found at {path} — is the app initialized?").
- Note `id` not found → MCP error result, not a panic.
- Empty `query` on `search_notes` → return recent notes (degraded but useful), or empty list; spec picks "return recent notes".
- `get_backlinks` with unknown title → empty list (not an error).
- Any handler error is mapped to a JSON-RPC error response; the server stays alive.

## Security

- **Read-only connection** = the process cannot write, even if a bug exists.
- **Local stdio only** — no listening socket, no remote access.
- Data is the user's own notebook dir; no credentials involved.

## Write/Delete Reservation (v2, architectural)

The design must allow adding create/update/delete/move tools without restructuring:

1. **Capability gating by connection mode.** `taxis-mcp` opens the DB via a configurable mode flag (default read-only). A future `--read-write` flag (or config) opens a RW connection with `busy_timeout` to coexist with the app under WAL. The same `NotebookService::create_note/update_note/delete_note/move_note` methods are then callable from new tool handlers — no new storage code.
2. **Tool handlers are thin wrappers** over `NotebookService` methods. Adding a `create_note` tool means: a new handler calling the existing service method. The pattern is established in v1.
3. **Write tools are simply not registered** in v1's `tools/list` until enabled — they can ship dormant in the code or be added later. Either way the dispatch layer is unchanged.
4. **Conflict consideration for v2:** the app and MCP both writing under WAL is safe (single-writer serialization), but the MCP must reload/re-read after the app changes state. Reads already hit live DB rows, so staleness is minimal; no cache layer is introduced in v1 to avoid serving stale data.

v1 implements none of this; it only ensures nothing in the architecture forecloses it.

## Configuration & Discovery

Provide documentation (and a future in-app helper) for wiring into clients:

```jsonc
// Claude Code / Codex mcpServers config
{
  "mcpServers": {
    "taxis": {
      "command": "/path/to/taxis-mcp"
      // optional: "args": ["--read-write"]   // v2
    }
  }
}
```

Build: `cargo build --bin taxis-mcp` (from `src-tauri`) produces the binary alongside the app. The data dir is auto-discovered; no args needed for v1.

A "Copy MCP config" button in the Taxis settings (showing the resolved binary path) is a **nice-to-have**, not v1 core.

## Testing

- **Unit tests** on `snippet.rs` (keyword window extraction, CJK handling, bounds).
- **Unit tests** on tool handlers using a temp data dir + seeded notes (the service methods are already pure functions of `&Database` / `&MarkdownStorage`).
- **Integration test:** spawn the binary, send `initialize` → `tools/list` → `tools/call search_notes`, assert well-formed JSON-RPC responses against a seeded temp notebook.
- Build check on every change: `cargo check --bin taxis-mcp`.

## MCP SDK Choice

Prefer the maintained **Rust MCP SDK** (e.g. `rmcp` / the `modelcontextprotocol` Rust SDK) if it is mature and tracks the current spec — it removes JSON-RPC and schema boilerplate. **Fallback:** hand-roll the four method handlers (initialize, tools/list, tools/call, resources/list+read) over `serde_json` line framing, which is small and avoids a dependency risk. Decision is finalized during planning by checking the SDK's current state.

## Out of Scope / Future

- Write/delete tools (v2 — reserved above).
- Multi-notebook / non-`default` notebooks.
- HTTP/SSE transport.
- Semantic / vector search (today's LIKE + FTS5 table; a `notes_fts` MATCH query is a possible v1.1 improvement, but not required for launch).
- Embeddings / RAG ranking.
