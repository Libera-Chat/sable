# Multi-stage Dockerfile for Sable IRC Server
# Stage 1: Build environment
FROM rust:1.83-slim AS builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y \
        pkg-config \
        libssl-dev \
        protobuf-compiler \
        libpq-dev \
        && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy source code
COPY . .

# Build the project in release mode
RUN cargo build --release --bins

# Stage 2: Runtime environment
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y \
        ca-certificates \
        curl \
        && rm -rf /var/lib/apt/lists/*

# Create sable user and directories
RUN useradd -r -s /bin/false -d /sable sable && \
    mkdir -p /sable/config /sable/certs /sable/data /sable/ipc && \
    chown -R sable:sable /sable

# Copy binaries from builder
COPY --from=builder /build/target/release/sable_ircd /usr/local/bin/
COPY --from=builder /build/target/release/listener_process /usr/local/bin/
COPY --from=builder /build/target/release/auth_client /usr/local/bin/

# Copy entrypoint script
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# Set working directory
WORKDIR /sable

# Switch to non-root user
USER sable

# Expose ports
# 6667/tcp - Plain text IRC
# 6697/tcp - TLS IRC
# 6668/tcp - Server-to-server gossip (TLS)
# 8888/tcp - Management HTTPS API
# 9999/tcp - Tokio console (debug)
EXPOSE 6667 6697 6668 8888 9999

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -fks https://localhost:8888/ || exit 1

# Set entrypoint and default command
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["-n", "/sable/config/network.conf", "-s", "/sable/config/server.conf", "--foreground"]
