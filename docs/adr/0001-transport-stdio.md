# ADR-0001: Transport stdio vs SSE

## Status

Accepted

## Context

MCP supports multiple transport mechanisms: stdio (JSON-RPC over stdin/stdout)
and Streamable HTTP (server-sent events). We need to choose one for gns3-mcp.

## Decision

Use **stdio** transport exclusively for v1.

## Consequences

- Native support in Claude Desktop and Claude Code without additional infrastructure
- Zero network setup: no HTTP server, no port binding, no TLS certificates
- Docker-friendly: `docker run --rm -i` maps stdio naturally
- Limitation: no multi-client access (one client per process). Acceptable for
  lab usage where one Claude session manages one GNS3 topology.
- SSE can be added later if needed (v2 scope).
