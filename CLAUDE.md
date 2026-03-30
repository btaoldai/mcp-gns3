---
tags:
  - claude/instructions
  - claude/dev
version: 1.0.0
created: 2026-03-30
project: gns3-mcp
---

# CLAUDE.md — gns3-mcp

> Root override for Claude Code. This file takes precedence over any generic instruction.
> Updated at each significant architectural decision.
> Orchestrator ref: section 3 (routing table) | version: 2026-03-30

---

## 0. Control center integration

This project is a **standalone** project attached to Baptiste's Obsidian vault.
It is not yet registered in `ORCHESTRATEUR.md` section 3 — to be done during
the first stable session.

### Parent vault context (read-only)

This CLAUDE.md is designed to be read by **Claude Code opened directly inside
`gns3-mcp/`**. The project lives inside the Obsidian vault `Vault-Pro` but Claude Code
only has access to the current directory. The references below are informational
to place the project within the ecosystem — they are not readable from this session.

**Full vault path**: `Vault-Pro/MCP/gns3-mcp/`

| Vault file (out of scope) | Role |
|---|---|
| `ORCHESTRATEUR.md` | Agent routing, project state |
| `.claude/context/mcp-servers.md` | Active MCP stack, transport debug |
| `.claude/backlogs/claude-friendly/backlog-centre-controle.md` | Current vault state |

### Hard rules (scope)

- Do not modify any file outside `gns3-mcp/`
- Do not touch any file marked `[LOCK]` in the Orchestrator
- Do not create logs in `.claude/logs/` — standalone project, not a BMAD agent
- Do not run a vault semantic scan to resolve Rust questions

### Position in the Orchestrator

- Paradigm: **Solo Dev (Barry)** — direct delivery, no full BMAD workflow
- Recommended model: **Sonnet** (code + infra)
- Model for structuring ADR decisions: **Opus (Winston)**
- To register in `ORCHESTRATEUR.md` section 3:
  `gns3-mcp | P2 | B (Build) | MCP/gns3-mcp/CLAUDE.md | ctx-centre-controle.md | Solo Dev (Barry) | Sonnet`

### Invariant collaboration rules (inherited from vault ROOT)

- **No commit / push / pull / rebase / merge without Baptiste's explicit approval**
- No emoji in produced files
- English for all produced content
- English for code and technical comments
- Validate before any structuring action
- Concise responses
- No secrets in plaintext outputs

---

## 1. Project context

Rust MCP server exposing the GNS3 v2 REST API to Claude via stdio transport.
Allows Claude to create, manipulate and supervise GNS3 network topologies
directly from a conversation (pedagogical use at YNOV / cybersecurity labs).

**v1 scope:**
- CRUD projects, nodes, links, templates
- Start/stop nodes
- Read topology state
- No network capture (Wireshark), no interactive console (v2)

**Stack:**
- Rust (multi-crate workspace, edition 2021)
- `rmcp` 1.3 — official Rust MCP SDK, stdio transport
- `reqwest` 0.12 + `rustls` — HTTP client, no glibc dependency
- `tokio` full — async runtime
- Docker multistage: `rust:1-alpine` (build) -> `distroless/static:nonroot` (runtime ~11 MB)

---

## 2. Workspace architecture

```
gns3-mcp/
├── Cargo.toml                  <- [workspace] + shared deps
├── CLAUDE.md                   <- this file
├── CHANGELOG.md
├── docs/adr/                   <- Architecture Decision Records
├── crates/
│   ├── core/                   <- types, traits, errors ONLY (0 network dependency)
│   ├── gns3-client/            <- HTTP implementation of Gns3Api
│   └── mcp-server/             <- MCP tools rmcp, main.rs, stdio bootstrap
└── Dockerfile
```

### Crate dependency rule (non-negotiable)

```
mcp-server  ->  core          (traits only)
mcp-server  ->  gns3-client   (injection in main.rs only)
gns3-client ->  core          (types + Gns3Api trait)
core        ->  nothing       (serde, thiserror, uuid only)
```

`mcp-server` must never import internal types from `gns3-client`.
Concrete wiring happens in `main.rs` only.

---

## 3. Code doctrine (non-negotiable)

### 3.1 Documentation first

- Every `pub` item has a `///` doc comment. No exception.
- Each `lib.rs` / `mod.rs` file has a `//!` module-level doc.
- Non-trivial public functions have an `# Examples` block.
- `CHANGELOG.md` updated with each significant feature or fix.
- Structuring decisions -> `docs/adr/NNNN-title.md`.

### 3.2 Zero Trust

- `GNS3_URL`, `GNS3_USER`, `GNS3_PASSWORD`: environment variables only.
  Never hardcoded, never logged, never in error messages.
- Credentials only flow through `Gns3ClientConfig` -> `Gns3Client`.
  `mcp-server` never sees them directly.
- Validate UUIDs received from Claude before any API call (newtypes if necessary).
- The binary runs as non-root user in Docker (`nonroot` distroless).

### 3.3 Modularity

- `core` is the only crate allowed to define shared types.
- `gns3-client` exposes only `Gns3Client` and `Gns3ClientConfig`.
- MCP tools are organized by domain in `mcp-server/src/tools/`:
  `projects.rs`, `nodes.rs`, `links.rs`, `templates.rs`.
- One file = one responsibility. No file > 300 lines without justification.

### 3.4 Resilience

- No `.unwrap()` outside `#[cfg(test)]`. Use `?`, `expect("explicit invariant")`.
- Every HTTP request has a timeout configured in `Gns3ClientConfig`.
- Graceful shutdown: `tokio::signal::ctrl_c()` + propagation via `watch` channel.
- MCP errors returned to Claude are actionable:
  state what failed AND what to do (e.g. "Project not found — list projects first").

---

## 4. Naming conventions

| Element | Convention | Example |
|---|---|---|
| Crate | kebab-case | `gns3-client` |
| Module | snake_case | `tools/nodes.rs` |
| Struct/Enum | PascalCase | `NodeStatus`, `Gns3Error` |
| Function | snake_case | `create_project` |
| MCP tool | snake_case | `gns3_create_project` |
| Constant | SCREAMING_SNAKE | `DEFAULT_TIMEOUT_SECS` |
| Env var | SCREAMING_SNAKE prefixed | `GNS3_URL` |

MCP tool naming: `gns3_` prefix + verb + entity.
Examples: `gns3_list_projects`, `gns3_create_node`, `gns3_start_node`.

---

## 5. MCP tools to implement

### Priority 1 — Foundation (do not move to P2 before these 8 tools are tested)

| Tool | GNS3 Endpoint | Description |
|---|---|---|
| `gns3_get_version` | `GET /v2/version` | Verifies that the GNS3 server is reachable |
| `gns3_list_projects` | `GET /v2/projects` | Lists all projects |
| `gns3_create_project` | `POST /v2/projects` | Creates a new project |
| `gns3_open_project` | `POST /v2/projects/{id}/open` | Opens a project (required before nodes) |
| `gns3_list_templates` | `GET /v2/templates` | Lists available templates |
| `gns3_create_node` | `POST /v2/projects/{id}/templates/{tid}` | Creates a node from a template |
| `gns3_start_node` | `POST /v2/projects/{id}/nodes/{nid}/start` | Starts a node |
| `gns3_create_link` | `POST /v2/projects/{id}/links` | Connects two interfaces |

### Priority 2 — Complete

| Tool | Description |
|---|---|
| `gns3_list_nodes` | Lists the nodes of a project with their status |
| `gns3_stop_node` | Stops a node |
| `gns3_delete_node` | Deletes a node |
| `gns3_list_links` | Lists the links of a project |
| `gns3_delete_link` | Deletes a link |
| `gns3_close_project` | Closes a project |
| `gns3_delete_project` | Deletes a project |
| `gns3_get_topology` | Composite tool: nodes + links in a single response |

### Priority 3 — V2 (do not implement now)

- Wireshark capture (pcap)
- Interactive console (Telnet)
- Snapshots
- Drawings / canvas

---

## 6. MCP response format

Tools return structured text readable by Claude, not raw JSON.

```
# Correct
Project created: "Lab-VLAN" (id: 3f2a1b...)
Status: opened | Nodes: 0

# Incorrect
{"project_id":"3f2a1b...","name":"Lab-VLAN","status":"opened"}
```

For lists: condensed markdown tables.
For errors: what failed + suggested corrective action.

---

## 7. Logging

- `tracing` exclusively. Never `println!` or `eprintln!` outside tests.
- The stdio transport uses stdout for the JSON-RPC protocol.
  Logs go to stderr — configure `tracing_subscriber` with `with_writer(std::io::stderr)`.
- Docker production: JSON format.
- Development: compact readable format.
- Level controlled by `RUST_LOG` (default: `info`).

```rust
// main.rs — mandatory before everything
tracing_subscriber::fmt()
    .with_writer(std::io::stderr)
    .with_env_filter(EnvFilter::from_default_env())
    .init();
```

---

## 8. Dockerfile (reference)

```dockerfile
# Stage 1: Build
FROM rust:1-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY . .
RUN cargo build --release --bin gns3-mcp

# Stage 2: Runtime
FROM gcr.io/distroless/static:nonroot
COPY --from=builder /app/target/release/gns3-mcp /usr/local/bin/gns3-mcp
ENTRYPOINT ["/usr/local/bin/gns3-mcp"]
```

Expected final image: ~11 MB.
Verify static compilation: `ldd target/release/gns3-mcp` -> "not a dynamic executable".

---

## 9. Claude Desktop / Claude Code configuration

```json
{
  "mcpServers": {
    "gns3": {
      "command": "docker",
      "args": [
        "run", "--rm", "-i",
        "-e", "GNS3_URL",
        "-e", "GNS3_USER",
        "-e", "GNS3_PASSWORD",
        "--network", "host",
        "gns3-mcp:latest"
      ],
      "env": {
        "GNS3_URL": "http://localhost:3080"
      }
    }
  }
}
```

**Credentials note**: `-e GNS3_USER` and `-e GNS3_PASSWORD` in `args` perform
**passthrough** from the host environment (Docker inherits variables if they
exist on the host side). They are intentionally not defined in `env` —
credentials must not appear in plaintext in the JSON config. Define them
in the host shell (`export GNS3_USER=...`) or via a Docker `.env` file.

`--network host` required if GNS3 runs locally.
For GNS3 on a remote VPS: put the full URL in `GNS3_URL`.

---

## 10. Tests

- Unit tests in each crate: `#[cfg(test)] mod tests { ... }`
- `gns3-client`: mock of the `Gns3Api` trait to test tools without a real GNS3 server
- Integration tests in `crates/mcp-server/tests/` against a real GNS3 (optional CI)
- Commands:

```
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

---

## 11. Existing ADRs

| # | Title | Decision |
|---|---|---|
| 0001 | stdio vs SSE transport | stdio — native to Claude Desktop, 0 infrastructure |
| 0002 | Alpine + distroless vs Debian | Alpine musl -> ~11 MB image, 0 glibc dep |
| 0003 | rustls vs openssl | rustls — friction-free musl compilation, no libssl |
| 0004 | Gns3Api trait in core | Decouples mcp-server / gns3-client, mockable in tests |

---

## 12. Hard rules

- Do not write `.unwrap()` or `.expect("")` empty outside tests
- Do not hardcode `localhost:3080` other than as a documented default value
- Do not log credentials (even partially)
- Do not add dependencies outside the workspace without justification in chat
- Do not implement Priority 3 features before Priority 1 is complete and tested
- Do not use `println!` in production code (stdout reserved for the MCP protocol)
- Do not make `core` depend on `gns3-client` or `mcp-server`
- Do not modify any file outside the `gns3-mcp/` directory
- Do not touch a `[LOCK]` file in the Obsidian vault
- Do not create logs in `.claude/logs/` (standalone project, outside BMAD scope)
