# ADR-0005 — Circuit Breaker in `core`

## Status

Proposed — 2026-03-31

## Context

`gns3-mcp` relies on an active GNS3 server for every tool call. When the server
is unavailable (crash, restart, network partition), the current retry helper
fires three attempts per call before returning an error. In a multi-user lab
environment (e.g. YNOV M1 class of 20+ students), all tools are called
concurrently. If GNS3 is down, each tool call generates 3 HTTP requests before
failing — producing a request storm that can delay GNS3 recovery.

Additionally, Claude will keep retrying tool sequences until it gets a response,
amplifying the storm.

## Decision

Implement a three-state async circuit breaker (`Closed → Open → Half-Open`) in
`crates/core/src/circuit_breaker.rs` and wire it into `gns3-client` so that all
outbound calls go through it.

Key parameters:

| Parameter | Default | Rationale |
|-----------|---------|-----------|
| `failure_threshold` | 5 | Tolerate transient 5xx bursts without opening |
| `recovery_timeout` | 30 s | Short enough for interactive labs |

The circuit breaker is placed in `core` (not `gns3-client`) so that it can be:
- Unit-tested without HTTP stubs
- Reused if a second client implementation is ever written
- Mocked independently in tool-level tests

## Consequences

**Accepted trade-offs:**

- A single `CircuitBreaker` instance is shared across all tool calls via
  `Arc<Mutex<State>>`. This adds one async lock acquisition per call. The lock
  is held only for state reads/writes (no I/O under the lock), so contention is
  negligible.
- The first call after the recovery timeout is a "probe" — it may fail, which
  keeps the circuit open for another `recovery_timeout`. This is intentional:
  it protects the GNS3 server from being hit before it is ready.

**Not solved by this ADR:**

- Per-project or per-node circuit isolation (out of scope for v0.2)
- Prometheus metrics for circuit state (tracked in a future ADR)
