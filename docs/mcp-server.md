# Taxa MCP Server

Taxa ships a read-only **MCP (Model Context Protocol)** server so that AI tools —
Claude Code, Codex, and any MCP-compatible client — can search and read your notes
as a local knowledge base.

It is a separate binary, `taxa-mcp`, that shares the app's storage code and reads
the same database + markdown files. It never writes.

## Build

From the repository root:

```bash
cargo build --release --bin taxa-mcp
```

The binary lands at `target/release/taxa-mcp` (`.exe` on Windows).

## Wire it into Claude Code

Add `taxa` to your MCP config (e.g. `~/.claude.json` → `mcpServers`, or via
`claude mcp add`). Point `command` at the built binary:

```jsonc
{
  "mcpServers": {
    "taxa": {
      "command": "/absolute/path/to/taxa-mcp"
    }
  }
}
```

No arguments are needed in v1 — it auto-discovers the data directory
(`{data_dir}/Taxis`) the same way the app does.

## Exposed capabilities

**Tools** (the AI calls these):

| Tool | What it does |
|------|--------------|
| `search_notes` | Keyword search across title/tags/content; returns id, title, folder, snippet |
| `read_note` | Full note content + metadata by id |
| `list_notes` | List a folder, or recent notes across all folders |
| `get_folder_tree` | Notebook folder structure with note counts |
| `get_backlinks` | Notes that `[[wikilink]]` to a given title |

**Resources**: `taxa://notes/{id}` → the note's markdown content.

Typical flow: `search_notes` to find relevant notes → `read_note` (or read the
resource) for full content. `get_backlinks` to traverse related notes via wikilinks.

## Notes

- **Read-only.** The server physically cannot create, edit, or delete notes.
  Write/delete support is reserved for a future version.
- **Concurrent with the app.** Opens the database read-side under WAL; safe to run
  while the Taxa desktop app is open.
- The database must already exist (open the Taxa app once to initialize it).

## Troubleshooting

- *Server exits immediately:* the database wasn't found. Open the Taxa desktop app
  once so it creates `{data_dir}/Taxis/taxis.db`. The server logs the expected path
  to stderr.
