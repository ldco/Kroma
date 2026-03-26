# Virus Scanning with ClamAV

File upload endpoints now include optional virus scanning using ClamAV to prevent malware uploads.

## Overview

- **Package**: `clamscan` (optional dependency)
- **Engine**: ClamAV antivirus daemon
- **Behavior**:
  - Enabled by default in production
  - Disabled by default in development
  - Fails open (allows uploads if ClamAV unavailable with logged warning)

## Installation

### 1. Install ClamAV

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install clamav clamav-daemon
```

**Arch Linux:**
```bash
sudo pacman -S clamav
```

**macOS:**
```bash
brew install clamav
```

### 2. Update Virus Definitions

```bash
# Stop daemon if running
sudo systemctl stop clamav-daemon

# Update definitions
sudo freshclam

# Start daemon
sudo systemctl start clamav-daemon
sudo systemctl enable clamav-daemon  # Enable on boot
```

### 3. Install Node Package

```bash
npm install clamscan
```

### 4. Verify Installation

Check that ClamAV daemon is running:
```bash
# Check status
sudo systemctl status clamav-daemon

# Test socket
ls -la /var/run/clamav/clamd.ctl
```

## Configuration

### Environment Variables

```bash
# Enable/disable scanning (default: true in production, false in dev)
CLAMAV_ENABLED=true

# Custom socket path (default: /var/run/clamav/clamd.ctl)
CLAMAV_SOCKET=/var/run/clamav/clamd.ctl
```

### Default Behavior

| Environment | Default | Can Override |
|-------------|---------|--------------|
| Production | Enabled | Set `CLAMAV_ENABLED=false` to disable |
| Development | Disabled | Set `CLAMAV_ENABLED=true` to enable |

## How It Works

1. **File Upload** → Magic byte validation → **Virus scan** → Storage
2. If virus detected → Upload rejected with 400 error
3. If ClamAV unavailable → Upload proceeds with warning logged
4. Scan happens in-memory for performance (uses daemon socket)

## Integrated Endpoints

Virus scanning is active on:
- `POST /api/upload/image` - Image uploads
- `POST /api/upload/video` - Video uploads

## Testing

### Test with EICAR File

```bash
# Download EICAR test file (harmless test virus)
curl https://secure.eicar.org/eicar.com -o eicar.com

# Try uploading via API (should be rejected)
curl -X POST http://localhost:3000/api/upload/image \
  -H "Cookie: pm-session=YOUR_SESSION" \
  -F "image=@eicar.com"

# Expected response:
# {"statusCode":400,"statusMessage":"File rejected: Eicar-Signature"}
```

## Troubleshooting

### Daemon Not Running

```bash
# Check logs
sudo journalctl -u clamav-daemon

# Common issue: Database outdated
sudo freshclam
sudo systemctl restart clamav-daemon
```

### Socket Permission Denied

```bash
# Add user to clamav group
sudo usermod -a -G clamav $USER

# Or update socket permissions
sudo chmod 666 /var/run/clamav/clamd.ctl
```

### High Memory Usage

ClamAV daemon uses ~500MB-1GB RAM. For low-memory systems:

1. Disable in development: `CLAMAV_ENABLED=false`
2. Use file-based scanning (slower):
   ```javascript
   // In virusScanning.ts, set:
   preference: 'clamscan'  // instead of 'clamdscan'
   ```

### Package Not Installed

If you don't want virus scanning:
```bash
# Set environment variable
CLAMAV_ENABLED=false

# Don't install clamscan package
# App will log warning but continue working
```

## Security Considerations

- ✅ **Fail-open design**: If ClamAV fails, uploads proceed with logged warning
- ✅ **Magic byte validation**: Always enforced regardless of ClamAV status
- ✅ **Auth required**: Only authenticated admins can upload
- ⚠️ **Not a silver bullet**: ClamAV has ~24hr lag for new threats
- ⚠️ **False positives**: Rare but possible (check logs)

## Performance

- **In-memory scan**: ~50-200ms per file (using daemon socket)
- **Memory**: Daemon uses ~500MB-1GB resident
- **CPU**: Minimal (daemon handles multiple requests efficiently)
- **Throughput**: ~10-50 files/second depending on size

## Production Recommendations

1. **Enable ClamAV** on production servers
2. **Update definitions** daily via cron:
   ```bash
   # Add to /etc/cron.daily/freshclam
   #!/bin/bash
   freshclam --quiet
   ```
3. **Monitor logs** for scan failures
4. **Alert on malware** detections:
   ```bash
   # Check logs for detections
   grep "Virus detected" /var/log/puppet-master/access.log
   ```

## References

- [ClamAV Documentation](https://docs.clamav.net/)
- [clamscan npm package](https://www.npmjs.com/package/clamscan)
- [EICAR Test File](https://www.eicar.org/download-anti-malware-testfile/)
