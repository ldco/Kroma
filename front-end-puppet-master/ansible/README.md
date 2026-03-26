# Ansible - Server Provisioning

Prepares Selectel VPS for PuppetMaster deployment with Kamal.

## Prerequisites

```bash
# Install Ansible
pip install ansible

# Or on Ubuntu/Debian
sudo apt install ansible
```

## Quick Start

1. **Create a VPS on Selectel** (Ubuntu 22.04 recommended)

2. **Configure inventory:**

   ```bash
   cd ansible
   cp inventory.example.yml inventory.yml
   # Edit inventory.yml with your server IP
   ```

3. **Run the playbook:**

   ```bash
   ansible-playbook playbook.yml
   ```

4. **Deploy with Kamal:**
   ```bash
   cd ..
   npm run deploy:setup
   npm run deploy
   ```

## What Gets Installed

| Component   | Purpose                      |
| ----------- | ---------------------------- |
| Docker      | Container runtime            |
| UFW         | Firewall (ports 22, 80, 443) |
| FFmpeg      | Video processing             |
| SQLite3     | Database CLI tools           |
| deploy user | Kamal deployment user        |

## Roles

- **common** - System packages, timezone, limits
- **docker** - Docker Engine with compose plugin
- **security** - UFW firewall, SSH hardening
- **deploy-user** - Create user for Kamal with Docker access

## Commands

```bash
# Full setup
ansible-playbook playbook.yml

# Only specific role
ansible-playbook playbook.yml --tags docker

# Specific hosts
ansible-playbook playbook.yml --limit production

# Check mode (dry run)
ansible-playbook playbook.yml --check
```
