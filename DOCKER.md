# Docker Deployment Guide for Sable IRC Server

This guide covers deploying the Sable IRC server using Docker and Docker Compose.

## Quick Start

### 1. Generate TLS Certificates

For production use, obtain proper TLS certificates. For testing, you can generate self-signed certificates:

```bash
cd docker/certs
openssl req -x509 -newkey rsa:4096 -keyout server.key -out server.crt -days 365 -nodes -subj "/CN=server1.example.com"
cp server.crt ca_cert.pem

# For client certificate (management API access)
openssl req -x509 -newkey rsa:4096 -keyout client.key -out client.crt -days 365 -nodes -subj "/CN=admin"

# Get fingerprints for configuration
openssl x509 -noout -fingerprint -sha1 -in server.crt
openssl x509 -noout -fingerprint -sha1 -in client.crt
```

### 2. Configure the Server

Copy the example configuration files and edit them:

```bash
cd docker/config
cp network.conf.example network.conf
cp server.conf.example server.conf
cp network_config.json.example network_config.json
cp motd.txt.example motd.txt
```

Edit the files and update:
- **network.conf**: Update peer fingerprints and addresses
- **server.conf**: Update `server_id`, `server_name`, and `authorised_fingerprints`
- **network_config.json**: Set operator password hash (`openssl passwd -6`)

### 3. Build and Start

```bash
# Build the Docker image
docker build -t sable-ircd .

# Start with Docker Compose
docker compose up -d

# View logs
docker compose logs -f
```

### 4. Verify

```bash
# Check container status
docker ps

# Test connection (plain text)
nc localhost 6667

# Test connection (TLS)
openssl s_client -connect localhost:6697

# Test management API (requires client certificate)
curl --cert docker/certs/client.crt --key docker/certs/client.key https://localhost:8888/
```

## Configuration

### Directory Structure

```
sable/
├── Dockerfile
├── docker-compose.yml
└── docker/
    ├── entrypoint.sh
    ├── certs/
    │   ├── server.crt       # Server TLS certificate
    │   ├── server.key       # Server private key
    │   ├── ca_cert.pem      # CA certificate (for server-to-server)
    │   ├── client.crt       # Management API client cert (optional)
    │   └── client.key       # Management API client key (optional)
    ├── config/
    │   ├── network.conf     # Network configuration
    │   ├── server.conf      # Server-specific configuration
    │   ├── network_config.json  # Bootstrap configuration
    │   └── motd.txt         # Message of the Day
    └── data/
        └── logs/            # Log files (mounted volume)
```

### Environment Variables

- `RUST_LOG`: Log level (default: `info`)
- `BOOTSTRAP_NETWORK`: Optional path to network bootstrap config
- `SABLE_NETWORK_CONF`: Path to network.conf (default: `/sable/config/network.conf`)
- `SABLE_SERVER_CONF`: Path to server.conf (default: `/sable/config/server.conf`)
- `SABLE_CERT_DIR`: Path to certificates directory (default: `/sable/certs`)

### Exposed Ports

| Port | Protocol | Description |
|------|----------|-------------|
| 6667 | TCP | Plain text IRC (development only) |
| 6697 | TCP | TLS-secured IRC |
| 6668 | TCP | Server-to-server gossip (TLS) |
| 8888 | TCP | Management HTTPS API |
| 9999 | TCP | Tokio console (debugging, optional) |

## Multi-Server Deployment

To run multiple servers in a network, uncomment the second service in `docker-compose.yml`:

```yaml
sable-ircd-2:
  # ... configuration ...
  environment:
    - RUST_LOG=info
```

Create `server2.conf` with a different `server_id` (e.g., `2`) and `server_name`, and update the peer configurations accordingly.

## Management API

The management API requires client certificate authentication. Access it with:

```bash
curl --cert docker/certs/client.crt \
     --key docker/certs/client.key \
     https://localhost:8888/api/status
```

## Health Checks

The container includes a built-in health check that queries the management API every 30 seconds. View health status:

```bash
docker ps  # Look under STATUS
docker inspect sable-ircd-1 | jq '.[0].State.Health'
```

## Persistence

Log files and state data are stored in the `docker/data` directory, which is mounted as a volume. This ensures data persists across container restarts.

## Troubleshooting

### Container won't start

Check the logs:
```bash
docker compose logs
```

Common issues:
- Missing configuration files
- Missing TLS certificates
- Incorrect file paths in configuration
- Port conflicts

### Can't connect to IRC

1. Check if ports are exposed: `docker ps`
2. Check firewall settings
3. Verify server is running: `docker compose logs`

### TLS errors

- Ensure certificates are mounted correctly
- Verify `ca_file` paths in configuration
- Check certificate fingerprints match in peer configuration
- Verify server certificate includes the correct hostname

### Certificate errors

Generate new certificates and update fingerprints:
```bash
# Get server fingerprint
openssl x509 -noout -fingerprint -sha1 -in docker/certs/server.crt

# Get client fingerprint
openssl x109 -noout -fingerprint -sha1 -in docker/certs/client.crt
```

Update these fingerprints in `network.conf` and `server.conf`.

## Development

### Development Override

Create `docker-compose.override.yml` for development:

```yaml
version: '3.8'
services:
  sable-ircd-1:
    environment:
      - RUST_LOG=debug
    volumes:
      - ./target:/build/target  # Mount build artifacts
      - ./sable_ircd:/build/sable_ircd  # Mount source for hot reload
```

### Building Locally

If you want to build locally before deploying:

```bash
cargo build --release
docker build --build-arg BUILD_TYPE=local -t sable-ircd .
```

## Security Considerations

1. **Production**: Disable plain text IRC (port 6667), use only TLS (6697)
2. **Certificates**: Use proper CA-signed certificates in production
3. **Management API**: Restrict access with firewall rules and strong client certificates
4. **Operator passwords**: Use strong password hashes
5. **File permissions**: Ensure private key files have `0600` permissions
6. **Debug mode**: Disable `debug_mode` in production configuration

## Upgrading

To upgrade to a new version:

```bash
docker compose down
docker build -t sable-ircd:latest .
docker compose up -d
```

The server supports hot-restart, which preserves active connections during upgrades.

## Support

For issues and questions:
- GitHub Issues: https://github.com/your-repo/sable
- Documentation: See main README.md
- IRC: Join #sable on your network
