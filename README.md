# gns3-mcp -- AI-Powered Network Engineering

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![MCP](https://img.shields.io/badge/MCP-stdio-green.svg)](https://modelcontextprotocol.io)
[![Tools](https://img.shields.io/badge/Tools-19-blueviolet.svg)](#available-tools-19)
[![Docker](https://img.shields.io/badge/Docker-~19%20MB-informational.svg)](#quick-start)

**gns3-mcp** is a high-performance MCP server written in **Rust** that exposes the **GNS3 REST API v2** to Claude, turning your AI assistant into a full-fledged network engineer -- designing, deploying and managing complex lab topologies through natural conversation.

---

## Why gns3-mcp?

Stop clicking through GUIs to build labs. With `gns3-mcp`, Claude can:

- **Design** entire topologies from a text description.
- **Deploy** and wire nodes (Cisco, Arista, Docker, VPCS...) in seconds.
- **Inspect** and document the live state of your lab.
- **Operate** the full device lifecycle -- start, stop, and reconfigure -- by voice or text.

Both you and Claude work on the same GNS3 project simultaneously: you adjust in the GUI while Claude deploys and wires from the conversation. Real-time, bidirectional.

---

## Highlights

- **Ultra-lightweight**: ~19 MB Docker image (`distroless/static`, musl static binary).
- **19 MCP tools**: full CRUD for projects, nodes, links, templates, compute servers, plus batch and composite operations.
- **Resilient**: automatic retry with exponential backoff on 5xx errors (3 attempts, 100/200/400 ms).
- **Secure by design**: zero OpenSSL/glibc dependency (rustls), UUID validation on every input, credentials never logged, non-root container.
- **Clean Architecture**: 3-crate Rust workspace with strict dependency inversion and trait-based injection.
- **Zero friction**: stdio transport -- native Claude Desktop and Claude Code integration, no extra infrastructure.

---

## Architecture

```
gns3-mcp/
+-- Cargo.toml              # workspace root + shared deps
+-- crates/
|   +-- core/               # types, traits, errors (zero network deps)
|   +-- gns3-client/        # HTTP client implementing Gns3Api
|   +-- mcp-server/         # MCP tools, stdio bootstrap
+-- docs/adr/               # Architecture Decision Records
+-- .github/workflows/      # CI + Docker pipelines
+-- Dockerfile              # multistage: Alpine build -> distroless runtime
```

The project enforces a strict dependency rule (Dependency Inversion):

```
mcp-server  -->  core          (traits only)
mcp-server  -.-> gns3-client   (injected in main.rs only)
gns3-client -->  core          (types + Gns3Api trait)
core        -->  nothing       (serde, thiserror, uuid)
```

> **Key insight**: `mcp-server` has zero knowledge of HTTP or reqwest. It interacts only through the `Gns3Api` trait defined in `core`, making every tool fully unit-testable with a mock -- no GNS3 server required.

---

## Quick Start

### Prerequisites

- GNS3 server 2.2.x+, running and accessible over HTTP
- Rust 1.75+ **or** Docker

### Option 1: Docker (recommended)

```bash
docker build -t gns3-mcp:latest .

docker run --rm -i \
  -e GNS3_URL=http://localhost:3080 \
  --network host \
  gns3-mcp:latest
```

### Option 2: Build from source

```bash
cargo build --release
GNS3_URL=http://localhost:3080 ./target/release/gns3-mcp
```

---

## Claude Desktop / Claude Code Configuration

Add to your config file (`claude_desktop_config.json` or `.claude/settings.json`):

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

Credentials use Docker pass-through (`-e VAR` without `=value`). Define them in your shell or `.env` file -- never in the JSON config.

### Environment variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `GNS3_URL` | Yes | `http://127.0.0.1:3080` | GNS3 server base URL |
| `GNS3_USER` | No | -- | Basic auth username |
| `GNS3_PASSWORD` | No | -- | Basic auth password |
| `GNS3_TIMEOUT_SECS` | No | `30` | HTTP request timeout in seconds |
| `RUST_LOG` | No | `info` | Log level (trace, debug, info, warn, error) |

---

## Available Tools (19)

### Projects

| Tool | What it does |
|---|---|
| `gns3_get_version` | Verify GNS3 server connectivity and version |
| `gns3_list_projects` | List all projects with status |
| `gns3_create_project` | Create and open a new project |
| `gns3_open_project` | Open an existing project (required before node/link ops) |
| `gns3_close_project` | Close a project, release resources |
| `gns3_delete_project` | Permanently delete a project |
| `gns3_get_topology` | Full snapshot: all nodes + all links in one call |

### Nodes

| Tool | What it does |
|---|---|
| `gns3_list_templates` | Discover available appliances (routers, switches, etc.) |
| `gns3_create_node` | Deploy a node from a template onto the canvas |
| `gns3_list_nodes` | List nodes with status, type, and console ports |
| `gns3_start_node` | Boot a single node |
| `gns3_stop_node` | Shut down a single node |
| `gns3_delete_node` | Remove a node from the project |
| `gns3_start_all_nodes` | Boot every node in the project at once |
| `gns3_stop_all_nodes` | Shut down every node in the project at once |

### Links

| Tool | What it does |
|---|---|
| `gns3_create_link` | Wire two node interfaces together |
| `gns3_list_links` | List all connections in the project |
| `gns3_delete_link` | Remove a connection |

### Infrastructure

| Tool | What it does |
|---|---|
| `gns3_list_computes` | List compute servers with CPU/memory usage |

### Typical workflow

```
1. gns3_get_version        -- verify connectivity
2. gns3_create_project     -- or gns3_list_projects to reuse one
3. gns3_open_project       -- unlock node and link operations
4. gns3_list_templates     -- discover available appliances
5. gns3_create_node        -- repeat for each device
6. gns3_create_link        -- repeat for each cable
7. gns3_start_all_nodes    -- boot everything at once
8. gns3_get_topology       -- review the live topology
```

---

## Resilience

- **Retry on 5xx**: exponential backoff (100 ms, 200 ms, 400 ms), up to 3 retries. 4xx and network errors fail immediately -- no wasted time on bad requests.
- **Configurable timeout**: `GNS3_TIMEOUT_SECS` env var, default 30 s.
- **Actionable errors**: every error returned to Claude explains what failed and suggests the next step.

---

## Security

| Property | Detail |
|---|---|
| **Credentials** | Environment variables only -- never logged, never in error messages, never visible to MCP |
| **Input validation** | Every UUID from Claude is validated before any API call |
| **Container** | `distroless/static:nonroot` -- no shell, no package manager, unprivileged user |
| **TLS** | Pure-Rust `rustls` -- no OpenSSL, no glibc dependency |
| **Binary** | Statically linked musl -- zero dynamic dependencies |

---

## Development

```bash
cargo check --workspace                    # type-check
cargo test --workspace                     # 39 unit tests, no GNS3 needed
cargo clippy --workspace -- -D warnings    # lint (zero warnings policy)
cargo fmt --check                          # format check
```

CI runs automatically on every push and pull request via GitHub Actions.

---

## Compatibility

gns3-mcp speaks standard **JSON-RPC over stdio** -- the open MCP protocol. Any client that implements MCP can use it, not just Claude.

| Client | Status | Notes |
|---|---|---|
| **Claude Desktop / Code** | Native | stdio supported out of the box |
| **OpenAI / ChatGPT** | Supported | MCP support announced March 2025 |
| **Cursor** | Native | Built-in MCP client |
| **Windsurf** | Native | Built-in MCP client |
| **Continue.dev** | Native | Bridges MCP to any LLM backend (Ollama, llama.cpp, etc.) |
| **LM Studio** | Via wrapper | Needs an MCP client layer (e.g. Python `mcp` SDK) |
| **Custom agents** | Yes | Any code that can spawn a subprocess and write JSON-RPC to stdin |

> **Note on model capability**: the quality of the experience depends on the LLM's ability to understand tool descriptions and sequence calls correctly (e.g. `open_project` before `create_node`). Large models (70B+, Claude, GPT-4) handle this well. Smaller local models (7-13B) may struggle with multi-step tool orchestration.

---

## Architecture Decisions

Design rationale is documented in `docs/adr/`:

| ADR | Decision |
|---|---|
| 0001 | **stdio over SSE** -- native Claude Desktop support, zero infrastructure |
| 0002 | **Alpine + distroless** -- musl static binary, ~19 MB final image |
| 0003 | **rustls over OpenSSL** -- no glibc friction for musl cross-compilation |
| 0004 | **Gns3Api trait in core** -- decouples server from client, enables mock testing |

---

## License

MIT -- see [LICENSE](LICENSE)
