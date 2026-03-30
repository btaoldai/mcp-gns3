# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
