# ADR-0004: Trait Gns3Api in core

## Status

Accepted

## Context

The MCP server needs to call GNS3 API operations. We want the server logic
to be testable without a real GNS3 instance, and we want clean dependency
boundaries between crates.

## Decision

Define the `Gns3Api` async trait in the `core` crate. The `gns3-client` crate
provides the concrete HTTP implementation. The `mcp-server` crate depends on
`core` for the trait and receives an `Arc<dyn Gns3Api>` via constructor injection.

## Consequences

- `mcp-server` has zero knowledge of HTTP/reqwest: clean separation of concerns
- Unit tests can mock `Gns3Api` without spinning up a GNS3 server
- `core` stays lightweight: only types, traits, and errors (no network deps)
- The only place that knows about the concrete `Gns3Client` is `main.rs`,
  where dependency injection happens
- Adding alternative backends (e.g., a cached client, a recording/replay
  client for tests) requires only implementing the trait.
