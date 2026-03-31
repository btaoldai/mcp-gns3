# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅ Yes    |

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Please report security issues by emailing the maintainer directly or by using
[GitHub's private vulnerability reporting](https://github.com/btaoldai/mcp-gns3/security/advisories/new).

Include the following in your report:

- A description of the vulnerability and its potential impact
- Steps to reproduce or a proof-of-concept
- Affected version(s)
- Any suggested mitigation

You can expect an acknowledgement within **72 hours** and a resolution timeline
within **14 days** for critical issues.

## Security Considerations

### Network exposure

`gns3-mcp` communicates with a GNS3 server over HTTP/HTTPS. The server is
typically localhost or a trusted LAN address. **Never expose a GNS3 server
directly to the public internet without authentication and TLS.**

### Docker network mode

The quick-start example uses `--network host` for convenience on Linux. This
gives the container full access to the host network stack and should only be
used in isolated lab environments.

**For production or shared environments**, use a named Docker network and pass
an explicit `GNS3_URL` pointing to the GNS3 server IP:

```bash
docker network create gns3-net

docker run --rm -i \
  --network gns3-net \
  -e GNS3_URL=http://gns3-server:3080 \
  gns3-mcp:latest
```

Or on a single host without host networking:

```bash
docker run --rm -i \
  --add-host host.docker.internal:host-gateway \
  -e GNS3_URL=http://host.docker.internal:3080 \
  gns3-mcp:latest
```

### Credentials

`GNS3_USER` and `GNS3_PASSWORD` are passed exclusively via environment
variables. They are **never** logged, never included in error messages, and
never visible to the MCP layer. Use Docker's pass-through syntax (`-e VAR`
without `=value`) to inherit them from the shell rather than embedding them
in compose files or CI configs.

### Input validation

All UUIDs received from the LLM are validated with the `uuid` crate before any
GNS3 API call. Invalid inputs are rejected immediately without touching the
network.

### Container hardening

The runtime image is `gcr.io/distroless/static:nonroot`:

- No shell, no package manager, no OS utilities
- Runs as an unprivileged user (`nonroot`)
- Statically linked musl binary — zero dynamic library dependencies
- No OpenSSL (uses `rustls`)
