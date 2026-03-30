# ADR-0002: Alpine + distroless vs Debian

## Status

Accepted

## Context

The Docker image must be small, secure, and quick to pull in lab environments
(student machines, VPS with limited bandwidth).

## Decision

Use `rust:1-alpine` for the build stage (musl libc, static linking) and
`gcr.io/distroless/static:nonroot` for the runtime stage.

## Consequences

- Final image ~19 MB (vs ~80-150 MB with Debian slim)
- Static binary: no runtime libc dependency, no shared library issues
- `nonroot` user: defense in depth, reduced blast radius if compromised
- No shell in runtime image: harder to debug live, but accepted trade-off
  (debug via `docker run --rm -it rust:1-alpine` with the binary mounted)
- musl may have minor performance differences vs glibc for some workloads;
  irrelevant for an MCP server doing HTTP API calls.
