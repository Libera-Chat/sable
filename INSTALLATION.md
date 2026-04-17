# VPS Installation Guide for Sable IRC Server

This guide covers deploying the Sable IRC server on a VPS (Virtual Private Server) using Docker.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [VPS Requirements](#vps-requirements)
3. [Initial Server Setup](#initial-server-setup)
4. [Docker Installation](#docker-installation)
5. [DNS Configuration](#dns-configuration)
6. [TLS Certificate Setup](#tls-certificate-setup)
7. [Sable Configuration](#sable-configuration)
8. [Firewall Configuration](#firewall-configuration)
9. [Deployment](#deployment)
10. [Post-Installation](#post-installation)
11. [Maintenance](#maintenance)
12. [Troubleshooting](#troubleshooting)

---

## Prerequisites

- A VPS with a Linux distribution (Ubuntu 22.04/24.04 or Debian 12 recommended)
- Root or sudo access
- A domain name pointed to your VPS
- Basic knowledge of SSH and command line

---

## VPS Requirements

### Minimum Specifications

- **CPU**: 2 cores
- **RAM**: 2 GB
- **Storage**: 20 GB SSD
- **Bandwidth**: 1 TB/month (for ~50 concurrent users)
- **Operating System**: Ubuntu 22.04/24.04 or Debian 12

### Recommended Specifications

- **CPU**: 4 cores
- **RAM**: 4 GB
- **Storage**: 40 GB SSD
- **Bandwidth**: 2 TB/month or more

---

## Initial Server Setup

### 1. Connect to Your VPS

```bash
ssh root@your-vps-ip
```

### 2. Update the System

```bash
# Ubuntu/Debian
apt update && apt upgrade -y
```

### 3. Create a Non-Root User (Recommended)

```bash
# Create user
adduser sable
usermod -aG sudo sable

# Set up SSH keys
mkdir -p /home/sable/.ssh
cp ~/.ssh/authorized_keys /home/sable/.ssh/
chown -R sable:sable /home/sable/.ssh
chmod 700 /home/sable/.ssh
chmod 600 /home/sable/.ssh/authorized_keys
```

### 4. Disable Root SSH Login (Optional but Recommended)

```bash
# Edit SSH config
nano /etc/ssh/sshd_config

# Change these settings:
PermitRootLogin no
PasswordAuthentication no

# Restart SSH
systemctl restart sshd
```

### 5. Set Timezone

```bash
timedatectl set-timezone UTC
```

### 6. Set Hostname

```bash
hostnamectl set-hostname irc.yourdomain.com
```

---

## Docker Installation

### Ubuntu/Debian

```bash
# Install prerequisites
apt update
apt install -y ca-certificates curl gnupg

# Add Docker's official GPG key
install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
chmod a+r /etc/apt/keyrings/docker.gpg

# Set up repository
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
  $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | \
  tee /etc/apt/sources.list.d/docker.list > /dev/null

# Install Docker
apt update
apt install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

# Enable and start Docker
systemctl enable docker
systemctl start docker

# Add your user to docker group (optional)
usermod -aG docker sable
```

### Verify Docker Installation

```bash
docker --version
docker compose version
```

---

## DNS Configuration

### Required DNS Records

| Type | Name | Value | TTL |
|------|------|-------|-----|
| A | irc | Your VPS IP | 3600 |
| A | *.irc | Your VPS IP | 3600 |

### Example for Cloudflare/Other DNS Providers

```
irc.yourdomain.com        A    123.45.67.89
*.irc.yourdomain.com      A    123.45.67.89
```

---

## TLS Certificate Setup

### Option 1: Let's Encrypt (Recommended for Production)

#### Install Certbot

```bash
apt install -y certbot
```

#### Generate Certificates

```bash
# Stop port 80/443 if needed
certbot certonly --standalone -d irc.yourdomain.com
```
Or use the DNS challenge instead
```bash
certbot certonly --dns-<provider> -d irc.yourdomain.com
```
Or use Cloudflare DNS if you have it
```bash
# Install the plugin
pip install certbot-dns-cloudflare

# Create creds file
mkdir -p /etc/letsencrypt
cat > /etc/letsencrypt/cloudflare.ini << EOF
dns_cloudflare_api_token = YOUR_CF_API_TOKEN
EOF
chmod 600 /etc/letsencrypt/cloudflare.ini

# Issue cert
certbot certonly \
  --dns-cloudflare \
  --dns-cloudflare-credentials /etc/letsencrypt/cloudflare.ini \
  -d irc.yourdomain.com
```
#### Copy Certificates to Project Directory

```bash
# Create directory
mkdir -p /home/sable/sable-docker/certs

# Copy certificates
cp /etc/letsencrypt/live/irc.yourdomain.com/fullchain.pem /home/sable/sable-docker/certs/server.crt
cp /etc/letsencrypt/live/irc.yourdomain.com/privkey.pem /home/sable/sable-docker/certs/server.key
cp /etc/letsencrypt/live/irc.yourdomain.com/chain.pem /home/sable/sable-docker/certs/ca_cert.pem

# Set permissions
chmod 644 /home/sable/sable-docker/certs/server.crt
chmod 600 /home/sable/sable-docker/certs/server.key
chmod 644 /home/sable/sable-docker/certs/ca_cert.pem
```

#### Get Certificate Fingerprint

```bash
openssl x509 -noout -fingerprint -sha1 -in /home/sable/sable-docker/certs/server.crt
```

#### Set Up Auto-Renewal

```bash
# Test renewal
certbot renew --dry-run

# Create renewal hook
cat > /etc/letsencrypt/renewal-hooks/post/restart-sable.sh << 'EOF'
#!/bin/bash
cp /etc/letsencrypt/live/irc.yourdomain.com/fullchain.pem /home/sable/sable-docker/certs/server.crt
cp /etc/letsencrypt/live/irc.yourdomain.com/privkey.pem /home/sable/sable-docker/certs/server.key
chmod 644 /home/sable/sable-docker/certs/server.crt
chmod 600 /home/sable/sable-docker/certs/server.key
cd /home/sable/sable-docker && docker compose restart
EOF

chmod +x /etc/letsencrypt/renewal-hooks/post/restart-sable.sh
```

### Option 2: Self-Signed Certificates (For Testing)

```bash
# Create directory
mkdir -p /home/sable/sable-docker/certs
cd /home/sable/sable-docker/certs

# Generate self-signed certificate
openssl req -x509 -newkey rsa:4096 -keyout server.key -out server.crt -days 365 -nodes -subj "/CN=irc.yourdomain.com"

# Use server.crt as CA certificate for simplicity
cp server.crt ca_cert.pem

# Get fingerprint
openssl x509 -noout -fingerprint -sha1 -in server.crt
```

### Generate Client Certificate (For Management API)

```bash
cd /home/sable/sable-docker/certs

# Generate client certificate
openssl req -x509 -newkey rsa:4096 -keyout client.key -out client.crt -days 365 -nodes -subj "/CN=admin"

# Get client fingerprint
openssl x509 -noout -fingerprint -sha1 -in client.crt
```

---

## Sable Configuration

### 1. Clone or Upload Project

```bash
# Option A: Clone from git (if available)
cd /home/sable
git clone https://github.com/gfnord/sable.git sable-docker
cd sable-docker

# Option B: Upload manually
# On your local machine:
tar czf sable-docker.tar.gz Dockerfile docker-compose.yml docker/
scp sable-docker.tar.gz sable@your-vps:/home/sable/

# On VPS:
cd /home/sable
tar xzf sable-docker.tar.gz
mv sable-docker sable-docker
cd sable-docker
```

### 2. Generate Operator Password Hash

```bash
# On VPS or any Linux machine
openssl passwd -6
# Enter your password and copy the output
```

### 3. Create Network Configuration

```bash
cd /home/sable/sable-docker/config
cp network.conf.example network.conf
nano network.conf
```

Edit `network.conf`:

```json
{
    "fanout": 2,
    "ca_file": "/sable/certs/ca_cert.pem",
    "peers": [
        {
            "name": "irc.yourdomain.com",
            "address": "sable-ircd-1:6668",
            "fingerprint": "PASTE_YOUR_SERVER_FINGERPRINT_HERE"
        }
    ]
}
```

### 4. Create Server Configuration

```bash
cp server.conf.example server.conf
nano server.conf
```

Edit `server.conf`:

```json
{
    "server_id": 1,
    "server_name": "irc.yourdomain.com",
    
    "management": {
        "address": "0.0.0.0:8888",
        "client_ca": "/sable/certs/ca_cert.pem",
        "authorised_fingerprints": [
            {
                "name": "admin",
                "fingerprint": "PASTE_YOUR_CLIENT_FINGERPRINT_HERE"
            }
        ]
    },
    
    "server": {
        "listeners": [
            {"address": "0.0.0.0:6667"},
            {"address": "0.0.0.0:6697", "tls": true}
        ],
        "motd": "/sable/config/motd.txt",
        "admin": {
            "server_location": "Your Location",
            "description": "Sable IRC Server",
            "email": "admin@yourdomain.com"
        }
    },
    
    "event_log": {
        "event_expiry": 300
    },
    
    "tls_config": {
        "key_file": "/sable/certs/server.key",
        "cert_file": "/sable/certs/server.crt"
    },
    
    "node_config": {
        "listen_addr": "0.0.0.0:6668",
        "cert_file": "/sable/certs/server.crt",
        "key_file": "/sable/certs/server.key"
    },
    
    "log": {
        "dir": "/sable/data/logs",
        "module-levels": {
            "tokio": "warn",
            "runtime": "warn",
            "rustls": "error",
            "tracing": "warn",
            "sable": "info",
            "": "info"
        },
        "targets": [
            {
                "target": "stdout",
                "level": "info",
                "modules": ["sable", "audit"]
            },
            {
                "target": {"filename": "sable.log"},
                "level": "info"
            },
            {
                "target": {"filename": "audit.log"},
                "category": "audit",
                "level": "info"
            }
        ]
    }
}
```

### 5. Create Network Bootstrap Configuration

```bash
cp network_config.json.example network_config.json
nano network_config.json
```

Edit `network_config.json`:

```json
{
    "object_expiry": 300,
    "pingout_duration": 240,
    "opers": [
        {
            "name": "yournick",
            "hash": "$6$PASTE_YOUR_PASSWORD_HASH_HERE"
        }
    ],
    "alias_users": [
        {
            "nick": "ChanServ",
            "user": "ChanServ",
            "host": "services.",
            "realname": "Channel services",
            "command_alias": "CS"
        },
        {
            "nick": "NickServ",
            "user": "NickServ",
            "host": "services.",
            "realname": "Account services",
            "command_alias": "NS"
        }
    ],
    "default_roles": {
        "builtin:op": [
            "always_send", "op_self", "op_grant", "voice_self", "voice_grant",
            "receive_op", "receive_voice", "topic", "kick", "set_simple_mode",
            "ban_view", "ban_add", "ban_remove_any", "invite_self"
        ],
        "builtin:voice": [
            "always_send", "voice_self", "receive_voice", "ban_view"
        ]
    },
    "debug_mode": false
}
```

### 6. Create MOTD File

```bash
cp motd.txt.example motd.txt
nano motd.txt
```

### 7. Set Proper Permissions

```bash
cd /home/sable/sable-docker
chmod 644 config/*.conf config/*.json config/motd.txt
chmod 600 certs/server.key certs/client.key
chown -R sable:sable /home/sable/sable-docker
```

---

## Firewall Configuration

### UFW (Uncomplicated Firewall) - Ubuntu/Debian

```bash
# Install UFW if not present
apt install -y ufw

# Allow SSH
ufw allow 22/tcp

# Allow IRC ports
ufw allow 6667/tcp comment 'IRC (plain)'
ufw allow 6697/tcp comment 'IRC (TLS)'

# Allow server-to-server gossip
ufw allow 6668/tcp comment 'Server gossip'

# Allow management API (optional, restrict to specific IP)
# ufw allow from YOUR_IP to any port 8888 proto tcp comment 'Management API'

# Enable firewall
ufw enable

# Check status
ufw status
```

### iptables (Alternative)

```bash
# Allow established connections
iptables -A INPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT

# Allow SSH
iptables -A INPUT -p tcp --dport 22 -j ACCEPT

# Allow IRC
iptables -A INPUT -p tcp --dport 6667 -j ACCEPT
iptables -A INPUT -p tcp --dport 6697 -j ACCEPT
iptables -A INPUT -p tcp --dport 6668 -j ACCEPT

# Allow management API (optional, restrict IP)
# iptables -A INPUT -s YOUR_IP -p tcp --dport 8888 -j ACCEPT

# Drop everything else
iptables -A INPUT -j DROP

# Save rules
apt install -y iptables-persistent
netfilter-persistent save
```

---

## Deployment

### 1. Build the Docker Image

```bash
cd /home/sable/sable-docker
docker compose build
```

### 2. Start the Server

```bash
docker compose up -d
```

### 3. Check Logs

```bash
# Follow logs
docker compose logs -f

# Check for errors
docker compose logs | grep -i error
```

### 4. Verify Container Status

```bash
docker compose ps
```

Expected output should show the container as "Up (healthy)".

### 5. Test Connection

```bash
# Test plain text connection
nc localhost 6667

# Test TLS connection
openssl s_client -connect localhost:6697 -servername irc.yourdomain.com
```

---

## Post-Installation

### 1. Connect with an IRC Client

Using your favorite IRC client (HexChat, irssi, WeeChat, etc.):

```
Server: irc.yourdomain.com
Port: 6697 (TLS) or 6667 (plain)
SSL/TLS: Yes
```

### 2. Register Your Account

```
/msg NickServ REGISTER password email@example.com
```

### 3. Oper Up (IRC Operator)

```
/OPER yournick password
```

### 4. Configure Management API Access

The management API requires client certificate authentication:

```bash
# On your local machine, copy the client cert and key
scp sable@your-vps:/home/sable/sable-docker/certs/client.crt ./
scp sable@your-vps:/home/sable/sable-docker/certs/client.key ./

# Test API access
curl --cert client.crt --key client.key https://irc.yourdomain.com:8888/
```

### 5. Set Up Log Rotation

Create `/etc/logrotate.d/sable-docker`:

```
/home/sable/sable-docker/data/logs/*.log {
    daily
    rotate 14
    compress
    delaycompress
    missingok
    notifempty
    create 0640 sable sable
    postrotate
        docker compose restart sable-ircd-1
    endscript
}
```

---

## Maintenance

### View Logs

```bash
# All logs
docker compose logs

# Follow logs
docker compose logs -f

# Specific service
docker compose logs -f sable-ircd-1

# Last 100 lines
docker compose logs --tail=100
```

### Restart the Server

```bash
# Restart container
docker compose restart

# Restart specific service
docker compose restart sable-ircd-1
```

### Update Sable

```bash
cd /home/sable/sable-docker

# Pull latest code (if using git)
git pull

# Rebuild
docker compose build

# Restart with new image
docker compose up -d
```

### Backup Configuration

```bash
# Create backup script
cat > /home/sable/backup-sable.sh << 'EOF'
#!/bin/bash
BACKUP_DIR="/home/sable/backups/$(date +%Y%m%d)"
mkdir -p "$BACKUP_DIR"
cp -r /home/sable/sable-docker/config "$BACKUP_DIR/"
cp -r /home/sable/sable-docker/certs "$BACKUP_DIR/"
tar czf "$BACKUP_DIR.tar.gz" "$BACKUP_DIR"
rm -rf "$BACKUP_DIR"
# Keep last 30 days
find /home/sable/backups -mtime +30 -delete
EOF

chmod +x /home/sable/backup-sable.sh

# Add to crontab (daily at 3 AM)
crontab -e
# Add: 0 3 * * * /home/sable/backup-sable.sh
```

### Monitor Disk Space

```bash
# Check disk usage
df -h

# Check Docker space usage
docker system df

# Clean up unused Docker resources
docker system prune -a
```

---

## Troubleshooting

### Container Won't Start

```bash
# Check logs
docker compose logs

# Verify configuration files exist
ls -la config/
ls -la certs/

# Test configuration syntax
cat config/network.conf | jq .
cat config/server.conf | jq .
```

### Can't Connect to IRC

1. **Check firewall:**
   ```bash
   ufw status
   iptables -L -n
   ```

2. **Check container is running:**
   ```bash
   docker compose ps
   ```

3. **Check ports are listening:**
   ```bash
   netstat -tlnp | grep -E '6667|6697|6668'
   ```

4. **Verify certificates:**
   ```bash
   openssl x509 -in certs/server.crt -text -noout
   ```

### TLS Errors

1. **Verify certificate matches domain:**
   ```bash
   openssl x509 -in certs/server.crt -noout | grep DNS
   ```

2. **Test TLS connection:**
   ```bash
   openssl s_client -connect irc.yourdomain.com:6697 -servername irc.yourdomain.com
   ```

3. **Check certificate expiration:**
   ```bash
   openssl x509 -in certs/server.crt -noout -dates
   ```

### Container Crashes on Restart

```bash
# Check for state issues
docker compose down
# Remove volumes (WARNING: deletes data)
# docker compose down -v
docker compose up -d
```

### High Memory Usage

```bash
# Check container resource usage
docker stats

# Add memory limit to docker-compose.yml:
# services:
#   sable-ircd-1:
#     deploy:
#       resources:
#         limits:
#           memory: 2G
```

### Can't Oper Up

1. **Check operator credentials in network_config.json:**
   ```bash
   cat config/network_config.json | jq .opers
   ```

2. **Verify password hash:**
   ```bash
   # Generate new hash
   openssl passwd -6
   ```

3. **Check logs for authentication errors:**
   ```bash
   docker compose logs | grep -i oper
   ```

---

## Security Hardening

### 1. Disable Plain Text IRC (Production Only)

Edit `server.conf` and remove the port 6667 listener:

```json
"server": {
    "listeners": [
        {"address": "0.0.0.0:6697", "tls": true}
    ]
}
```

Update firewall:
```bash
ufw delete allow 6667/tcp
```

### 2. Restrict Management API to Specific IPs

```bash
ufw delete allow 8888/tcp
ufw allow from YOUR_IP to any port 8888 proto tcp
```

### 3. Enable Fail2Ban (Optional)

```bash
apt install -y fail2ban

# Create jail configuration
cat > /etc/fail2ban/jail.local << 'EOF'
[sable-irc]
enabled = true
port = 6667,6697
filter = sable-irc
logpath = /home/sable/sable-docker/data/logs/sable.log
maxretry = 5
bantime = 3600
EOF

# Create filter
cat > /etc/fail2ban/filter.d/sable-irc.conf << 'EOF'
[Definition]
failregex = .*Unauthorized connection from <HOST>
ignoreregex =
EOF

systemctl enable fail2ban
systemctl start fail2ban
```

---

## Getting Help

- **Documentation**: See `DOCKER.md` for additional Docker-specific information
- **GitHub Issues**: https://github.com/your-repo/sable/issues
- **IRC**: Join #sable on Libera.Chat for community support

---

## Quick Reference

### Essential Commands

```bash
# Start
docker compose up -d

# Stop
docker compose down

# Restart
docker compose restart

# Logs
docker compose logs -f

# Update
git pull && docker compose build && docker compose up -d

# Backup
tar czf backup-$(date +%Y%m%d).tar.gz config/ certs/

# Certificate renewal
certbot renew
```

### File Locations

- Config: `/home/sable/sable-docker/config/`
- Certificates: `/home/sable/sable-docker/certs/`
- Logs: `/home/sable/sable-docker/data/logs/`
- Docker Compose: `/home/sable/sable-docker/docker-compose.yml`

### Default Ports

- 6667: IRC (plain text)
- 6697: IRC (TLS)
- 6668: Server-to-server gossip
- 8888: Management API
- 9999: Tokio console (debug)

---

**Last Updated**: 2025-04-17
**Sable Version**: 0.1.0
