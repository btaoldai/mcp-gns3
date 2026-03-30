# Stage 1 : Build
FROM rust:1-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY . .
RUN cargo build --release --bin gns3-mcp

# Stage 2 : Runtime
FROM gcr.io/distroless/static:nonroot
COPY --from=builder /app/target/release/gns3-mcp /usr/local/bin/gns3-mcp
ENTRYPOINT ["/usr/local/bin/gns3-mcp"]
