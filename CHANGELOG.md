# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-03-31

### Security

* Added `SECURITY.md` with vulnerability reporting policy, network exposure
  guidance, and container hardening notes
* Added explicit warning for `--network host` Docker usage with safer
  `--add-host` alternative documented in README and SECURITY.md
* Added `cargo-deny` (`deny.toml`) to CI: blocks crates with known CVEs,
  GPL-licensed dependencies, and OpenSSL transitive pulls
* Blocked `openssl` and `openssl-sys` in `deny.toml` — enforces rustls-only
  TLS across the dependency tree

### Added

* Circuit breaker wired into `gns3-client`: all outbound calls now go through a
  three-state circuit breaker (`Closed → Open → Half-Open`) that fails fast
  after 5 consecutive failures and auto-recovers after 30s
* `crates/core/src/circuit_breaker.rs`: async three-state circuit breaker
  (`Closed → Open → Half-Open`) with configurable failure threshold and
  recovery timeout
* `CircuitBreakerError::Open` variant allows `mcp-server` to return an
  actionable message to the LLM when GNS3 is temporarily unavailable
* `Gns3Error::CircuitOpen` variant in `crates/core/src/error.rs`
* `CONTRIBUTING.md`: development workflow, branch strategy, commit convention,
  code standards checklist, and tool-addition guide
* `docs/adr/0005-circuit-breaker.md`: rationale for circuit breaker placement
  in `core`
* `docs/adr/0006-non-exhaustive-errors.md`: rationale for `#[non_exhaustive]`
  on public error enums
* CI workflow: added `cargo-deny` job (security + license audit) and Docker
  build smoke test

### Changed

* All public error enums in `crates/core/src/error.rs` are now
  `#[non_exhaustive]` — adding variants in future minor releases is no longer
  a breaking change
* Error enum variants restructured: `Http { status, message }`,
  `Network(String)`, `InvalidUuid(String)`, `Server(String)` — improves
  semantic clarity for different error categories

### Fixed

* (None — this is a non-breaking feature release)

## [0.1.0] - 2026-03-30

Initial release -- 19 MCP tools, Docker image, CI/CD.

### Added

- Workspace multi-crate: `core`, `gns3-client`, `mcp-server`
- `Gns3Api` async trait in `core` with 18 operations
- `Gns3Client` HTTP implementation with reqwest + rustls
- `Gns3Server` MCP server with 19 tools via rmcp 1.3 stdio transport
- Priority 1 tools: `gns3_get_version`, `gns3_list_projects`, `gns3_create_project`,
  `gns3_open_project`, `gns3_list_templates`, `gns3_create_node`,
  `gns3_start_node`, `gns3_create_link`
- Priority 2 tools: `gns3_list_nodes`, `gns3_stop_node`, `gns3_delete_node`,
  `gns3_list_links`, `gns3_delete_link`, `gns3_close_project`,
  `gns3_delete_project`, `gns3_get_topology` (composite)
- Batch tools: `gns3_start_all_nodes`, `gns3_stop_all_nodes`
- Infrastructure tool: `gns3_list_computes`
- Retry logic with exponential backoff on 5xx errors (3 retries, 100/200/400ms)
- Configurable timeout via `GNS3_TIMEOUT_SECS` env var (default 30s)
- 39 unit tests with `MockGns3Api` and `MockGns3ApiError`
- Dockerfile multistage (Alpine build, distroless runtime ~19 MB)
- `.dockerignore` for build context optimization
- GitHub Actions CI workflow (check, test, clippy, fmt, Docker build)
- GitHub Actions Docker workflow (build on `v*` tags)
- ADRs 0001-0004 (stdio, Alpine, rustls, Gns3Api trait)
- README.md, LICENSE (MIT)
