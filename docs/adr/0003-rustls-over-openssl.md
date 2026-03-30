# ADR-0003: rustls vs openssl

## Status

Accepted

## Context

The HTTP client needs TLS for connecting to remote GNS3 servers. Two main
options: OpenSSL (via `openssl-sys` crate) or rustls (pure Rust TLS).

## Decision

Use **rustls** via `reqwest`'s `rustls-tls` feature.

## Consequences

- No dependency on `libssl-dev` / `openssl`: clean musl/Alpine builds
- No C compilation required for TLS: faster CI, simpler Dockerfile
- Smaller attack surface: rustls is memory-safe by construction
- WebPKI root certificates bundled via `webpki-roots` crate
- Limitation: no client certificate auth (not needed for GNS3 basic auth)
- If GNS3 server uses a custom CA, we may need to add certificate loading
  later (rustls supports this via `rustls-native-certs`).
