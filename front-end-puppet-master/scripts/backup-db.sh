#!/bin/bash
# ============================================
# SQLite Database Backup Script
# PuppetMaster2
# ============================================
#
# Usage:
#   ./scripts/backup-db.sh              # Local backup only
#   ./scripts/backup-db.sh --remote     # Local + S3 upload
#   ./scripts/backup-db.sh --verify     # Verify backup integrity
#
# Environment variables:
#   DATABASE_URL      - Path to SQLite database (default: /app/data/sqlite.db)
#   BACKUP_DIR        - Local backup directory (default: /app/data/backups)
#   RETENTION_DAYS    - Days to keep local backups (default: 7)
#
# For S3 upload (requires aws cli):
#   AWS_ACCESS_KEY_ID     - S3 access key
#   AWS_SECRET_ACCESS_KEY - S3 secret key
#   S3_BUCKET             - S3 bucket name
#   S3_ENDPOINT           - S3 endpoint (for Backblaze B2, etc.)
#   S3_RETENTION_DAYS     - Days to keep S3 backups (default: 30)
#
# For backup encryption:
#   BACKUP_ENCRYPTION_KEY - Encryption key for backup files (required for --encrypt)
#                           Must be a strong passphrase for AES-256 encryption
#
# Recommended cron:
#   # Daily backup at 2 AM with S3 upload
#   0 2 * * * /app/scripts/backup-db.sh --remote >> /var/log/backup.log 2>&1
#   # Weekly full backup on Sunday
#   0 3 * * 0 /app/scripts/backup-db.sh --remote --weekly >> /var/log/backup.log 2>&1
# ============================================

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
DB_PATH="${DATABASE_URL:-/app/data/sqlite.db}"
BACKUP_DIR="${BACKUP_DIR:-/app/data/backups}"
RETENTION_DAYS="${RETENTION_DAYS:-7}"
S3_RETENTION_DAYS="${S3_RETENTION_DAYS:-30}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
DATE_TAG=$(date +%Y%m%d)

# Parse arguments
UPLOAD_TO_S3=false
VERIFY_BACKUP=false
WEEKLY_BACKUP=false
ENCRYPT_BACKUP=false

for arg in "$@"; do
    case $arg in
        --remote)
            UPLOAD_TO_S3=true
            ;;
        --verify)
            VERIFY_BACKUP=true
            ;;
        --weekly)
            WEEKLY_BACKUP=true
            ;;
        --encrypt)
            ENCRYPT_BACKUP=true
            ;;
    esac
done

# If uploading to S3, require encryption key for security
if [ "$UPLOAD_TO_S3" = true ] && [ -n "${BACKUP_ENCRYPTION_KEY:-}" ]; then
    ENCRYPT_BACKUP=true
fi

# Backup filename
if [ "$WEEKLY_BACKUP" = true ]; then
    BACKUP_FILE="backup_weekly_${TIMESTAMP}.db"
else
    BACKUP_FILE="backup_daily_${TIMESTAMP}.db"
fi

log() {
    echo -e "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

log_success() {
    echo -e "${GREEN}[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS: $1${NC}"
}

log_error() {
    echo -e "${RED}[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}"
}

log_warn() {
    echo -e "${YELLOW}[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}"
}

# Ensure backup directory exists
mkdir -p "$BACKUP_DIR"

log "Starting database backup..."
log "Database: $DB_PATH"
log "Backup dir: $BACKUP_DIR"

# Check if database exists
if [ ! -f "$DB_PATH" ]; then
    log_error "Database not found at $DB_PATH"
    exit 1
fi

# Create backup using SQLite's backup command (WAL-safe)
log "Creating backup..."
if ! sqlite3 "$DB_PATH" ".backup '$BACKUP_DIR/$BACKUP_FILE'"; then
    log_error "SQLite backup command failed"
    exit 1
fi

# Verify backup integrity
log "Verifying backup integrity..."
if ! sqlite3 "$BACKUP_DIR/$BACKUP_FILE" "PRAGMA integrity_check;" | grep -q "ok"; then
    log_error "Backup integrity check failed!"
    rm -f "$BACKUP_DIR/$BACKUP_FILE"
    exit 1
fi
log_success "Backup integrity verified"

# Compress backup
log "Compressing backup..."
gzip "$BACKUP_DIR/$BACKUP_FILE"
BACKUP_PATH="$BACKUP_DIR/${BACKUP_FILE}.gz"

# Encrypt backup if requested
if [ "$ENCRYPT_BACKUP" = true ]; then
    if [ -z "${BACKUP_ENCRYPTION_KEY:-}" ]; then
        log_error "BACKUP_ENCRYPTION_KEY not set. Cannot encrypt backup."
        exit 1
    fi

    log "Encrypting backup..."
    ENCRYPTED_PATH="${BACKUP_PATH}.enc"

    # Encrypt using OpenSSL AES-256-CBC with PBKDF2 key derivation
    if ! openssl enc -aes-256-cbc -salt -pbkdf2 -iter 100000 \
        -in "$BACKUP_PATH" \
        -out "$ENCRYPTED_PATH" \
        -pass env:BACKUP_ENCRYPTION_KEY; then
        log_error "Encryption failed!"
        rm -f "$ENCRYPTED_PATH"
        exit 1
    fi

    # Remove unencrypted backup
    rm -f "$BACKUP_PATH"
    BACKUP_PATH="$ENCRYPTED_PATH"
    log_success "Backup encrypted"
fi

# Get backup size
BACKUP_SIZE=$(du -h "$BACKUP_PATH" | cut -f1)
log_success "Backup created: $BACKUP_PATH ($BACKUP_SIZE)"

# Upload to S3 if requested
if [ "$UPLOAD_TO_S3" = true ]; then
    if [ -z "${S3_BUCKET:-}" ]; then
        log_warn "S3_BUCKET not set, skipping remote upload"
    else
        log "Uploading to S3..."

        # Build aws cli command
        AWS_CMD="aws s3 cp"
        if [ -n "${S3_ENDPOINT:-}" ]; then
            AWS_CMD="$AWS_CMD --endpoint-url $S3_ENDPOINT"
        fi

        # Use appropriate extension based on encryption
        if [ "$ENCRYPT_BACKUP" = true ]; then
            S3_PATH="s3://${S3_BUCKET}/backups/${DATE_TAG}/${BACKUP_FILE}.gz.enc"
        else
            S3_PATH="s3://${S3_BUCKET}/backups/${DATE_TAG}/${BACKUP_FILE}.gz"
        fi

        if $AWS_CMD "$BACKUP_PATH" "$S3_PATH"; then
            log_success "Uploaded to $S3_PATH"

            # Cleanup old S3 backups
            log "Cleaning up old S3 backups (older than $S3_RETENTION_DAYS days)..."
            CUTOFF_DATE=$(date -d "-${S3_RETENTION_DAYS} days" +%Y%m%d 2>/dev/null || date -v-${S3_RETENTION_DAYS}d +%Y%m%d)

            # List and delete old backups
            if [ -n "${S3_ENDPOINT:-}" ]; then
                aws s3 ls "s3://${S3_BUCKET}/backups/" --endpoint-url "$S3_ENDPOINT" 2>/dev/null | while read -r line; do
                    FOLDER_DATE=$(echo "$line" | awk '{print $2}' | tr -d '/')
                    if [[ "$FOLDER_DATE" =~ ^[0-9]{8}$ ]] && [ "$FOLDER_DATE" -lt "$CUTOFF_DATE" ]; then
                        log "Deleting old backup folder: $FOLDER_DATE"
                        aws s3 rm "s3://${S3_BUCKET}/backups/${FOLDER_DATE}/" --recursive --endpoint-url "$S3_ENDPOINT" 2>/dev/null || true
                    fi
                done
            else
                aws s3 ls "s3://${S3_BUCKET}/backups/" 2>/dev/null | while read -r line; do
                    FOLDER_DATE=$(echo "$line" | awk '{print $2}' | tr -d '/')
                    if [[ "$FOLDER_DATE" =~ ^[0-9]{8}$ ]] && [ "$FOLDER_DATE" -lt "$CUTOFF_DATE" ]; then
                        log "Deleting old backup folder: $FOLDER_DATE"
                        aws s3 rm "s3://${S3_BUCKET}/backups/${FOLDER_DATE}/" --recursive 2>/dev/null || true
                    fi
                done
            fi
        else
            log_error "S3 upload failed"
        fi
    fi
fi

# Cleanup old local backups (both encrypted and unencrypted)
log "Cleaning up local backups older than $RETENTION_DAYS days..."
find "$BACKUP_DIR" -name "backup_*.db.gz" -mtime +"$RETENTION_DAYS" -delete
find "$BACKUP_DIR" -name "backup_*.db.gz.enc" -mtime +"$RETENTION_DAYS" -delete

# Count remaining backups
BACKUP_COUNT=$(find "$BACKUP_DIR" \( -name "backup_*.db.gz" -o -name "backup_*.db.gz.enc" \) 2>/dev/null | wc -l)
log "Local backups retained: $BACKUP_COUNT"

# Verify backup if requested
if [ "$VERIFY_BACKUP" = true ]; then
    log "Performing full backup verification..."

    # Create temp dir for verification
    VERIFY_DIR=$(mktemp -d)
    VERIFY_DB="$VERIFY_DIR/verify.db"
    VERIFY_GZ="$VERIFY_DIR/verify.db.gz"

    # Decrypt if encrypted
    if [ "$ENCRYPT_BACKUP" = true ]; then
        if [ -z "${BACKUP_ENCRYPTION_KEY:-}" ]; then
            log_error "BACKUP_ENCRYPTION_KEY not set. Cannot verify encrypted backup."
            rm -rf "$VERIFY_DIR"
            exit 1
        fi

        log "Decrypting backup for verification..."
        if ! openssl enc -aes-256-cbc -d -pbkdf2 -iter 100000 \
            -in "$BACKUP_PATH" \
            -out "$VERIFY_GZ" \
            -pass env:BACKUP_ENCRYPTION_KEY; then
            log_error "Decryption failed!"
            rm -rf "$VERIFY_DIR"
            exit 1
        fi
        gunzip -c "$VERIFY_GZ" > "$VERIFY_DB"
    else
        # Decompress directly
        gunzip -c "$BACKUP_PATH" > "$VERIFY_DB"
    fi

    # Run integrity check
    if sqlite3 "$VERIFY_DB" "PRAGMA integrity_check;" | grep -q "ok"; then
        log_success "Full verification passed"

        # Count tables
        TABLE_COUNT=$(sqlite3 "$VERIFY_DB" "SELECT COUNT(*) FROM sqlite_master WHERE type='table';")
        log "Tables in backup: $TABLE_COUNT"
    else
        log_error "Full verification failed!"
        rm -rf "$VERIFY_DIR"
        exit 1
    fi

    rm -rf "$VERIFY_DIR"
fi

log_success "Backup complete."
