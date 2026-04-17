#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

echo_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

echo_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Configuration paths
NETWORK_CONF="${SABLE_NETWORK_CONF:-/sable/config/network.conf}"
SERVER_CONF="${SABLE_SERVER_CONF:-/sable/config/server.conf}"
NETWORK_CONFIG="${SABLE_NETWORK_CONFIG:-/sable/config/network_config.json}"
CERT_DIR="${SABLE_CERT_DIR:-/sable/certs}"

# Check if configuration files exist
check_config() {
    echo_info "Checking configuration files..."

    if [ ! -f "${NETWORK_CONF}" ]; then
        echo_error "Network configuration not found: ${NETWORK_CONF}"
        echo_error "Please mount or copy network.conf to this location"
        exit 1
    fi

    if [ ! -f "${SERVER_CONF}" ]; then
        echo_error "Server configuration not found: ${SERVER_CONF}"
        echo_error "Please mount or copy server.conf to this location"
        exit 1
    fi

    if [ ! -f "${NETWORK_CONFIG}" ]; then
        echo_error "Network bootstrap config not found: ${NETWORK_CONFIG}"
        echo_error "Please mount or copy network_config.json to this location"
        exit 1
    fi

    echo_info "Configuration files found"
}

# Check if certificates exist
check_certs() {
    echo_info "Checking TLS certificates..."

    local cert_missing=false

    if [ ! -f "${CERT_DIR}/server.crt" ]; then
        echo_warn "Server certificate not found: ${CERT_DIR}/server.crt"
        cert_missing=true
    fi

    if [ ! -f "${CERT_DIR}/server.key" ]; then
        echo_warn "Server key not found: ${CERT_DIR}/server.key"
        cert_missing=true
    fi

    if [ ! -f "${CERT_DIR}/ca_cert.pem" ]; then
        echo_warn "CA certificate not found: ${CERT_DIR}/ca_cert.pem"
        cert_missing=true
    fi

    if [ "$cert_missing" = true ]; then
        echo_error "Missing TLS certificates. Please provide certificates in ${CERT_DIR}:"
        echo_error "  - server.crt  : Server TLS certificate"
        echo_error "  - server.key  : Server private key"
        echo_error "  - ca_cert.pem : CA certificate for server-to-server authentication"
        echo ""
        echo_error "You can generate self-signed certificates for testing:"
        echo_error "  openssl req -x509 -newkey rsa:4096 -keyout server.key -out server.crt -days 365 -nodes"
        echo_error "  cp server.crt ca_cert.pem"
        exit 1
    fi

    echo_info "TLS certificates found"
}

# Check certificate permissions
check_permissions() {
    echo_info "Checking file permissions..."

    if [ -f "${CERT_DIR}/server.key" ]; then
        local perms=$(stat -c %a "${CERT_DIR}/server.key" 2>/dev/null || stat -f %A "${CERT_DIR}/server.key")
        if [ "$perms" != "600" ] && [ "$perms" != "400" ]; then
            echo_warn "Server key has loose permissions: ${perms} (recommended: 600)"
        fi
    fi
}

# Display server info
display_info() {
    echo_info "Starting Sable IRC Server..."
    echo_info "Network config: ${NETWORK_CONF}"
    echo_info "Server config: ${SERVER_CONF}"
    echo_info "Network bootstrap: ${NETWORK_CONFIG}"
    echo_info "Certificate directory: ${CERT_DIR}"
}

# Handle signals for graceful shutdown
trap 'echo_info "Received shutdown signal, stopping..."; exit 0' SIGTERM SIGINT

# Run checks
check_config
check_certs
check_permissions
display_info

# Build command arguments
ARGS=()
ARGS+=("-n" "${NETWORK_CONF}")
ARGS+=("-s" "${SERVER_CONF}")
ARGS+=("--foreground")

# Add bootstrap argument if it exists
if [ -n "${BOOTSTRAP_NETWORK}" ]; then
    ARGS+=("--bootstrap-network" "${BOOTSTRAP_NETWORK}")
fi

# Pass through any additional arguments
if [ $# -gt 0 ]; then
    ARGS+=("$@")
fi

# Start the server
echo_info "Executing: /usr/local/bin/sable_ircd ${ARGS[*]}"
exec /usr/local/bin/sable_ircd "${ARGS[@]}"
