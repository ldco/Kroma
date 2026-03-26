# PuppetMaster2 - Deployment Guide

## Overview

This guide covers deploying PuppetMaster2 to a production server using:

- **Ansible** - Server provisioning (one-time setup)
- **Docker** - Application containerization
- **Kamal** - Zero-downtime deployments
- **Traefik** - Reverse proxy with automatic SSL

```
┌─────────────────────────────────────────────────────────────────┐
│                      DEPLOYMENT FLOW                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   1. PROVISION          2. SETUP           3. DEPLOY            │
│   ───────────          ─────────          ─────────             │
│   ansible-playbook  →  kamal setup   →   kamal deploy           │
│                                                                  │
│   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐        │
│   │ Install     │    │ Start       │    │ Build image │        │
│   │ Docker      │    │ Traefik     │    │ Push to     │        │
│   │ Create user │    │ Configure   │    │ registry    │        │
│   │ Firewall    │    │ SSL certs   │    │ Deploy      │        │
│   └─────────────┘    └─────────────┘    └─────────────┘        │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Prerequisites

### Local Machine Requirements

```bash
# Node.js 20+
node --version  # v20.x.x

# Ruby 3.2+ (for Kamal)
ruby --version  # ruby 3.2.x

# Ansible
ansible --version  # ansible 2.15+

# Docker (for local testing)
docker --version
```

### Install Required Tools

```bash
# Install Kamal
gem install kamal

# Install Ansible Docker collection
ansible-galaxy collection install community.docker
```

### Server Requirements

- Ubuntu 22.04 LTS (recommended)
- 1GB+ RAM
- Root SSH access (initial setup only)
- Domain pointing to server IP

---

## Step 1: Configure Inventory

Copy and configure the Ansible inventory:

```bash
cd ansible
cp inventory.example.yml inventory.yml
```

Edit `inventory.yml`:

```yaml
all:
  vars:
    ansible_python_interpreter: /usr/bin/python3
    deploy_user: deploy
    deploy_user_ssh_key: "{{ lookup('file', '~/.ssh/id_rsa.pub') }}"

  children:
    production:
      hosts:
        prod-1:
          ansible_host: 123.45.67.89 # Your server IP
          domain: example.com # Your domain
```

---

## Step 2: Provision Server

Run Ansible playbook to prepare the server:

```bash
# Full provisioning (first time)
ansible-playbook playbook.yml -i inventory.yml

# Or target specific server
ansible-playbook playbook.yml -i inventory.yml --limit prod-1

# Or run specific roles
ansible-playbook playbook.yml -i inventory.yml --tags docker
ansible-playbook playbook.yml -i inventory.yml --tags security
```

### What Ansible Does

| Role          | Actions                                                            |
| ------------- | ------------------------------------------------------------------ |
| `common`      | Updates packages, installs essentials (curl, git, ffmpeg, sqlite3) |
| `security`    | Configures UFW firewall, fail2ban, SSH hardening                   |
| `docker`      | Installs Docker, creates Traefik network and volumes               |
| `deploy-user` | Creates `deploy` user with Docker permissions                      |

### Verify Server Setup

```bash
# SSH as deploy user
ssh deploy@123.45.67.89

# Check Docker
docker --version
docker network ls  # Should show traefik-public
docker volume ls   # Should show puppetmaster2_data, puppetmaster2_uploads

# Check firewall
sudo ufw status
```

---

## Step 3: Configure Kamal (Single Source of Truth)

**IMPORTANT:** All deployment values are configured in ONE file: `.kamal/secrets`

The `deploy.yml` file uses ERB templates to read from `.kamal/secrets`, so you only need to configure your values once.

### Create Kamal Secrets

```bash
# Copy secrets template
cp .kamal/secrets.example .kamal/secrets

# Edit with your values
vim .kamal/secrets
```

### Required Values in `.kamal/secrets`:

```bash
# ─────────────────────────────────────────────
# INFRASTRUCTURE (Required)
# ─────────────────────────────────────────────
SITE_DOMAIN=example.com          # Your domain
SERVER_IP=123.45.67.89           # Your server IP
ACME_EMAIL=admin@example.com     # For SSL certificates

# ─────────────────────────────────────────────
# CONTAINER REGISTRY (Required)
# ─────────────────────────────────────────────
DOCKER_REGISTRY=ghcr.io
GITHUB_USERNAME=your-username
KAMAL_REGISTRY_USERNAME=your-username
KAMAL_REGISTRY_PASSWORD=ghp_your_token

# ─────────────────────────────────────────────
# APPLICATION (Required)
# ─────────────────────────────────────────────
DATABASE_URL=/app/data/sqlite.db

# ─────────────────────────────────────────────
# OPTIONAL FEATURES
# ─────────────────────────────────────────────
SMTP_HOST=smtp.yandex.ru
SMTP_PORT=465
SMTP_USER=your@email.com
SMTP_PASS=your-password
SMTP_FROM=Your Site <noreply@example.com>

TELEGRAM_BOT_TOKEN=your-token
TELEGRAM_CHAT_ID=your-chat-id

# Uptime Kuma monitoring (optional)
# UPTIME_KUMA_SUBDOMAIN=status
```

### How It Works

The `deploy.yml` uses ERB interpolation to read these values:

```yaml
# deploy.yml reads from .kamal/secrets automatically:
servers:
  web:
    hosts:
      - <%= ENV.fetch('SERVER_IP') %>  # Reads SERVER_IP from secrets
    labels:
      traefik.http.routers.puppetmaster2.rule: Host(`<%= ENV.fetch('SITE_DOMAIN') %>`)
```

**You never need to edit `deploy.yml`** - just configure `.kamal/secrets` once!

---

## Step 4: Initial Deployment

### First-Time Setup

```bash
# Setup Kamal on server (installs Traefik, configures SSL)
kamal setup
```

### Deploy Application

```bash
# Build, push, and deploy
kamal deploy
```

### Verify Deployment

```bash
# Check deployment status
kamal details

# View logs
kamal logs

# Check health
curl https://example.com/api/health
```

---

## Regular Operations

### Deploy New Version

```bash
# Deploy latest code
kamal deploy

# Deploy specific git ref
kamal deploy --version=v1.2.3
```

### Rollback

```bash
# Rollback to previous version
kamal rollback

# Rollback to specific version
kamal rollback v1.2.2
```

### View Logs

```bash
# Tail logs
kamal logs -f

# Last 100 lines
kamal logs -n 100

# Grep logs
kamal logs | grep ERROR
```

### SSH to Container

```bash
# Open shell in running container
kamal app exec -i bash

# Run single command
kamal app exec "node -v"
```

### Restart Application

```bash
# Restart app containers
kamal app boot
```

---

## Database Operations

### Backup Database

```bash
# SSH to server
ssh deploy@123.45.67.89

# Backup SQLite database
docker run --rm \
  -v puppetmaster2_data:/data \
  -v $(pwd):/backup \
  alpine \
  cp /data/sqlite.db /backup/sqlite-$(date +%Y%m%d-%H%M%S).db
```

### Restore Database

```bash
# Copy backup to server
scp sqlite-backup.db deploy@123.45.67.89:~/

# SSH to server and restore
ssh deploy@123.45.67.89
docker run --rm \
  -v puppetmaster2_data:/data \
  -v $(pwd):/backup \
  alpine \
  cp /backup/sqlite-backup.db /data/sqlite.db

# Restart app
kamal app boot
```

### Run Migrations

Migrations run automatically on startup. For manual migrations:

```bash
kamal app exec "npx drizzle-kit push"
```

---

## Troubleshooting

### Container Won't Start

```bash
# Check container status
kamal details

# View startup logs
kamal logs -n 200

# Check health endpoint locally in container
kamal app exec "wget -qO- http://localhost:3000/api/health"
```

### SSL Certificate Issues

```bash
# SSH to server
ssh deploy@123.45.67.89

# Check Traefik logs
docker logs traefik

# Verify Let's Encrypt storage
ls -la /letsencrypt/

# Force certificate renewal (recreate Traefik)
kamal traefik reboot
```

### Database Locked

```bash
# Check for WAL files
kamal app exec "ls -la /app/data/"

# If corrupted, restore from backup
```

### Out of Disk Space

```bash
# SSH to server
ssh deploy@123.45.67.89

# Check disk usage
df -h

# Clean Docker
docker system prune -a --volumes

# Check data volumes
docker system df -v
```

---

## CI/CD Automation

### GitHub Actions Deployment

To enable automated deployments on push to master:

1. Configure GitHub repository secrets:

| Secret                    | Description                      | Example                    |
| ------------------------- | -------------------------------- | -------------------------- |
| `DEPLOY_SSH_KEY`          | Private SSH key for deploy user  | `-----BEGIN OPENSSH...`    |
| `SERVER_IP`               | Target server IP (= .kamal/secrets) | `123.45.67.89`          |
| `SITE_DOMAIN`             | Domain (= .kamal/secrets)        | `example.com`              |
| `ACME_EMAIL`              | SSL cert email                   | `admin@example.com`        |
| `KAMAL_REGISTRY_USERNAME` | Docker registry username         | `your-github-username`     |
| `KAMAL_REGISTRY_PASSWORD` | Docker registry token            | `ghp_xxxx`                 |

2. Uncomment the deploy job in `.github/workflows/ci.yml`

The deploy job uses the same variables as `.kamal/secrets`, maintaining single source of truth.

### Manual Trigger

```bash
# From local machine
npm run deploy

# Or directly
kamal deploy
```

---

## Security

### SSH Access

SSH access is secured via key-based authentication only. Password login is disabled.

```bash
# Connect as deploy user
ssh deploy@your-server-ip

# Connect as root (if needed)
ssh root@your-server-ip
```

**Note**: You still have full SSH access. The security configuration only:

- Disables password authentication (requires SSH key)
- Disables root password login (key-only for root)
- Enables fail2ban to block brute force attempts

### SSH Audit Logging

SSH sessions are logged with verbose detail for security auditing:

```bash
# View SSH audit logs on server
sudo tail -f /var/log/auth.log | grep sshd

# Example output:
# Dec 20 14:32:15 server sshd[1234]: Accepted publickey for deploy from 1.2.3.4
# Dec 20 14:32:15 server sshd[1234]: pam_unix(sshd:session): session opened
```

Logged information includes:

- Login attempts (success and failure)
- SSH key fingerprint used
- Session start/end times
- Source IP addresses

### Firewall Rules (UFW)

Only essential ports are open:

| Port | Service     | Purpose               |
| ---- | ----------- | --------------------- |
| 22   | SSH         | Server access         |
| 80   | HTTP        | Redirect to HTTPS     |
| 443  | HTTPS       | Application traffic   |

```bash
# Check firewall status on server
sudo ufw status
```

### fail2ban Protection

Blocks IPs after repeated failed SSH attempts:

- **Max retries**: 5 attempts
- **Ban time**: 1 hour
- **Find time**: 10 minutes

```bash
# Check banned IPs on server
sudo fail2ban-client status sshd
```

### Security Checklist

- [ ] SSH key-based auth only (no passwords)
- [ ] Firewall enabled (UFW)
- [ ] fail2ban active
- [ ] Non-root container user
- [ ] Secrets in `.kamal/secrets` (not in git)
- [ ] HTTPS enforced via Traefik
- [ ] Regular backups scheduled
- [ ] SSH audit logging enabled (LogLevel VERBOSE)

---

## Monitoring

### Health Check Endpoint

```
GET /api/health

Response:
{
  "status": "healthy",
  "timestamp": "2024-12-20T12:00:00Z",
  "database": "connected"
}
```

### Log Monitoring

Logs are JSON-formatted (Pino). Use your preferred log aggregator:

```bash
# View structured logs
kamal logs | jq .

# Filter errors
kamal logs | jq 'select(.level >= 50)'
```

### Uptime Monitoring

Set up external monitoring (UptimeRobot, Pingdom, etc.) to ping:

- `https://example.com/api/health`

---

## Quick Reference

| Command                         | Description                 |
| ------------------------------- | --------------------------- |
| `ansible-playbook playbook.yml` | Provision server            |
| `kamal setup`                   | Initial Kamal/Traefik setup |
| `kamal deploy`                  | Deploy application          |
| `kamal rollback`                | Rollback to previous        |
| `kamal logs -f`                 | Tail logs                   |
| `kamal details`                 | Show deployment info        |
| `kamal app exec bash`           | Shell into container        |
| `kamal traefik reboot`          | Restart Traefik             |

---

## Architecture

```
                    ┌─────────────────────────────────────────┐
                    │              VPS Server                  │
                    │                                          │
    HTTPS:443 ──────┼──► ┌─────────────┐                      │
                    │    │   Traefik   │ ◄── Let's Encrypt    │
    HTTP:80  ───────┼──► │  (reverse   │                      │
    (redirect)      │    │   proxy)    │                      │
                    │    └──────┬──────┘                      │
                    │           │                              │
                    │           ▼                              │
                    │    ┌─────────────┐                      │
                    │    │ PuppetMaster│                      │
                    │    │  Container  │                      │
                    │    │  (port 3000)│                      │
                    │    └──────┬──────┘                      │
                    │           │                              │
                    │    ┌──────┴──────┐                      │
                    │    │   Volumes   │                      │
                    │    ├─────────────┤                      │
                    │    │ sqlite.db   │                      │
                    │    │ uploads/    │                      │
                    │    └─────────────┘                      │
                    │                                          │
                    └─────────────────────────────────────────┘
```
