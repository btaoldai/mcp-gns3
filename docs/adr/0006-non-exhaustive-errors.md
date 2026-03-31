# ADR-0006 — `#[non_exhaustive]` on Public Error Enums

## Status

Proposed — 2026-03-31

## Context

`gns3-mcp-core` exposes error enums (e.g. `Gns3Error`) as part of its public
API. Without `#[non_exhaustive]`, adding a new variant is a **breaking change**
for any downstream code that matches exhaustively on the enum.

Even though this project currently has no downstream consumers beyond its own
crates, marking errors `#[non_exhaustive]` from v0.1 avoids a future semver
major bump when new error conditions are discovered (e.g. authentication
failures, rate limiting, GNS3 v3 API changes).

## Decision

Annotate all public error enums in `crates/core/src/error.rs` with
`#[non_exhaustive]`.

```rust
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Gns3Error {
    #[error("HTTP error {status}: {message}")]
    Http { status: u16, message: String },

    #[error("Network error: {0}")]
    Network(String),

    #[error("Invalid UUID: {0}")]
    InvalidUuid(String),

    #[error("GNS3 server error: {0}")]
    Server(String),
}
```

Consumers (including `gns3-client` and `mcp-server`) must add a wildcard arm
to their `match` expressions:

```rust
match err {
    Gns3Error::Http { status, message } => { ... }
    Gns3Error::Network(msg) => { ... }
    _ => { /* future variants */ }
}
```

## Consequences

- Adding new variants in minor releases remains non-breaking.
- The `_` arm in `match` is required by the compiler — this serves as a
  reminder to handle new errors explicitly when upgrading `core`.
- `thiserror` is fully compatible with `#[non_exhaustive]`.
