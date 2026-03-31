# Contributing to gns3-mcp

Thank you for your interest in contributing. This document covers the workflow,
code standards, and review process.

---

## Quick start

```bash
git clone https://github.com/btaoldai/mcp-gns3.git
cd mcp-gns3

# Verify the workspace compiles and all tests pass
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

No GNS3 server is required — all 39 unit tests run against `MockGns3Api`.

---

## Branch strategy

| Branch | Purpose |
|--------|---------|
| `main` | Stable, releasable |
| `feat/<name>` | New features |
| `fix/<name>` | Bug fixes |
| `chore/<name>` | Tooling, deps, CI |
| `docs/<name>` | Documentation only |

Open a pull request against `main`. One PR per concern.

---

## Commit convention

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add gns3_update_node tool
fix: retry logic not applying 400ms delay on third attempt
docs: clarify --network host risk in SECURITY.md
refactor: extract retry helper into gns3-client/src/retry.rs
test: add circuit-breaker open-state test
chore: bump rmcp to 1.4
```

The `CHANGELOG.md` is updated **with the code**, not after the merge.

---

## Code standards

This project follows the [baptiste-code-style](https://github.com/btaoldai/mcp-gns3)
doctrine. Key rules:

### Documentation first

- Every `pub` item must have a `///` doc comment explaining the **why**.
- Every `lib.rs` / `mod.rs` starts with a `//!` module-level doc.
- Non-trivial public functions include an `# Examples` block.

### Zero Trust

- No `.unwrap()` in production code — use `?`, `.ok_or()`, or a documented
  `.expect("invariant reason")`.
- Secrets (credentials, tokens) must never appear in logs, error messages, or
  test fixtures.
- All inputs at crate boundaries are validated types (newtypes or explicit
  validation).

### Modularity

- `core` must stay free of network dependencies (`reqwest`, `tokio` net).
- `mcp-server` must not import `gns3-client` except in `main.rs`.
- New domains get their own crate, not an extra module in an existing one.

### Resilience

- Any new outbound HTTP call goes through the retry helper in `gns3-client`.
- When a circuit-breaker module lands, all outbound calls must respect it.

---

## Adding a new MCP tool

1. **Add the operation to `Gns3Api` in `crates/core/src/traits.rs`.**
   Document the method with `///`.

2. **Implement it in `crates/gns3-client/src/client.rs`.**
   Re-use the existing retry helper.

3. **Add a mock implementation in `crates/core/src/mock.rs`.**

4. **Register the tool in `crates/mcp-server/src/server.rs`.**
   Follow the existing `#[tool]` annotation pattern.

5. **Write at least two unit tests** (happy path + error path) in the
   `#[cfg(test)]` block of the relevant file.

6. **Update `CHANGELOG.md`** under `## [Unreleased]`.

7. **Update `README.md` and `README-fr.md`** tool tables.

---

## Pull request checklist

- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo fmt --check` passes
- [ ] Every new `pub` item has a doc comment
- [ ] `CHANGELOG.md` updated
- [ ] README tables updated if tools were added/changed
- [ ] No `.unwrap()` in new production code
- [ ] No secrets in test fixtures or examples

---

## Architecture Decision Records

Significant design choices are documented in `docs/adr/`. If your PR changes
the architecture (new crate, transport change, auth model), add an ADR.

Template: copy `docs/adr/0001-stdio-over-sse.md` and fill in **Context**,
**Decision**, and **Consequences**.

---

## Reporting bugs

Open a [GitHub issue](https://github.com/btaoldai/mcp-gns3/issues) with:

- gns3-mcp version and GNS3 server version
- Minimal reproduction steps
- Expected vs. actual behaviour
- Relevant log output (`RUST_LOG=debug`)

For security issues, see [SECURITY.md](SECURITY.md).
