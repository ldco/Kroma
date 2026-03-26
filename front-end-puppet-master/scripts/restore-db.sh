#!/bin/bash
# ============================================
# SQLite Database Restore Script
# PuppetMaster2
# ============================================
#
# Usage:
#   ./scripts/restore-db.sh                          # Interactive: list backups
#   ./scripts/restore-db.sh <backup_file>            # Restore specific backup
#   ./scripts/restore-db.sh --latest                 # Restore latest backup
#   ./scripts/restore-db.sh --from-s3 <s3_path>      # Restore from S3
#
# Environment variables:
#   DATABASE_URL      - Path to SQLite database (default: /app/data/sqlite.db)
#   BACKUP_DIR        - Local backup directory (default: /app/data/backups)
#
# For S3 restore:
#   AWS_ACCESS_KEY_ID     - S3 access key
#   AWS_SECRET_ACCESS_KEY - S3 secret key
#   S3_BUCKET             - S3 bucket name
#   S3_ENDPOINT           - S3 endpoint (for Backblaze B2, etc.)
#
# For encrypted backups:
#   BACKUP_ENCRYPTION_KEY - Decryption key for encrypted backup files (.gz.enc)
#                           Must match the key used during backup
#
# ============================================

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DB_PATH="${DATABASE_URL:-/app/data/sqlite.db}"
BACKUP_DIR="${BACKUP_DIR:-/app/data/backups}"

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

log_info() {
    echo -e "${BLUE}[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $1${NC}"
}

# List available backups
list_backups() {
    log_info "Available local backups:"
    echo ""
    if [ -d "$BACKUP_DIR" ]; then
        # List both encrypted (.gz.enc) and unencrypted (.gz) backups
        ls -lht "$BACKUP_DIR"/backup_*.db.gz* 2>/dev/null | head -20 || echo "No backups found in $BACKUP_DIR"
    else
        echo "Backup directory $BACKUP_DIR does not exist"
    fi
}

# Get latest backup
get_latest_backup() {
    if [ -d "$BACKUP_DIR" ]; then
        # Get latest backup (encrypted or unencrypted)
        ls -t "$BACKUP_DIR"/backup_*.db.gz* 2>/dev/null | head -1
    fi
}

# Check if file is encrypted
is_encrypted() {
    local file="$1"
    [[ "$file" == *.enc ]]
}

# Decrypt a backup file
decrypt_backup() {
    local encrypted_file="$1"
    local output_file="$2"

    if [ -z "${BACKUP_ENCRYPTION_KEY:-}" ]; then
        log_error "BACKUP_ENCRYPTION_KEY not set. Cannot decrypt backup."
        return 1
    fi

    log "Decrypting backup..."
    if ! openssl enc -aes-256-cbc -d -pbkdf2 -iter 100000 \
        -in "$encrypted_file" \
        -out "$output_file" \
        -pass env:BACKUP_ENCRYPTION_KEY; then
        log_error "Decryption failed! Check your BACKUP_ENCRYPTION_KEY."
        return 1
    fi

    log_success "Backup decrypted"
    return 0
}

# Verify backup file
verify_backup() {
    local backup_file="$1"
    local temp_db=$(mktemp)
    local temp_gz=$(mktemp)

    log "Verifying backup integrity..."

    # Handle encrypted backups
    if is_encrypted "$backup_file"; then
        if ! decrypt_backup "$backup_file" "$temp_gz"; then
            rm -f "$temp_db" "$temp_gz"
            return 1
        fi
        gunzip -c "$temp_gz" > "$temp_db"
        rm -f "$temp_gz"
    elif [[ "$backup_file" == *.gz ]]; then
        gunzip -c "$backup_file" > "$temp_db"
    else
        cp "$backup_file" "$temp_db"
    fi

    # Check integrity
    if sqlite3 "$temp_db" "PRAGMA integrity_check;" | grep -q "ok"; then
        log_success "Backup integrity verified"
        rm -f "$temp_db"
        return 0
    else
        log_error "Backup integrity check failed!"
        rm -f "$temp_db"
        return 1
    fi
}

# Stop application (if running)
stop_application() {
    log_warn "Stopping application..."
    # Try to gracefully stop the application
    # This depends on your deployment setup
    if command -v pm2 &> /dev/null; then
        pm2 stop all 2>/dev/null || true
    fi
    # Give it a moment
    sleep 2
}

# Start application
start_application() {
    log "Starting application..."
    if command -v pm2 &> /dev/null; then
        pm2 start all 2>/dev/null || true
    fi
}

# Restore from backup file
restore_from_file() {
    local backup_file="$1"

    if [ ! -f "$backup_file" ]; then
        log_error "Backup file not found: $backup_file"
        exit 1
    fi

    log "Restoring from: $backup_file"

    # Verify backup first
    if ! verify_backup "$backup_file"; then
        exit 1
    fi

    # Create a backup of current database before restore
    if [ -f "$DB_PATH" ]; then
        local pre_restore_backup="${DB_PATH}.pre-restore.$(date +%Y%m%d_%H%M%S)"
        log "Creating pre-restore backup: $pre_restore_backup"
        cp "$DB_PATH" "$pre_restore_backup"
    fi

    # Stop application
    stop_application

    # Remove WAL and SHM files if they exist
    rm -f "${DB_PATH}-wal" "${DB_PATH}-shm"

    # Decrypt (if needed), decompress, and restore
    log "Restoring database..."
    if is_encrypted "$backup_file"; then
        local temp_gz=$(mktemp)
        if ! decrypt_backup "$backup_file" "$temp_gz"; then
            log_error "Failed to decrypt backup"
            rm -f "$temp_gz"
            exit 1
        fi
        gunzip -c "$temp_gz" > "$DB_PATH"
        rm -f "$temp_gz"
    elif [[ "$backup_file" == *.gz ]]; then
        gunzip -c "$backup_file" > "$DB_PATH"
    else
        cp "$backup_file" "$DB_PATH"
    fi

    # Verify restored database
    if sqlite3 "$DB_PATH" "PRAGMA integrity_check;" | grep -q "ok"; then
        log_success "Database restored successfully"

        # Show some stats
        TABLE_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM sqlite_master WHERE type='table';")
        log_info "Tables in database: $TABLE_COUNT"

        # Run migrations if needed
        log_info "You may need to run 'npm run db:migrate' if there are pending migrations"
    else
        log_error "Restored database integrity check failed!"
        if [ -f "$pre_restore_backup" ]; then
            log_warn "Restoring from pre-restore backup..."
            cp "$pre_restore_backup" "$DB_PATH"
        fi
        exit 1
    fi

    # Start application
    start_application

    log_success "Restore complete!"
}

# Restore from S3
restore_from_s3() {
    local s3_path="$1"

    if [ -z "${S3_BUCKET:-}" ] && [[ ! "$s3_path" == s3://* ]]; then
        log_error "S3_BUCKET not set and path is not a full S3 URI"
        exit 1
    fi

    # Build full S3 path if needed
    if [[ ! "$s3_path" == s3://* ]]; then
        s3_path="s3://${S3_BUCKET}/$s3_path"
    fi

    log "Downloading from S3: $s3_path"

    # Create temp file with appropriate extension
    local temp_suffix=".db.gz"
    if [[ "$s3_path" == *.enc ]]; then
        temp_suffix=".db.gz.enc"
    fi
    local temp_backup=$(mktemp --suffix=$temp_suffix)

    # Download from S3
    local AWS_CMD="aws s3 cp"
    if [ -n "${S3_ENDPOINT:-}" ]; then
        AWS_CMD="$AWS_CMD --endpoint-url $S3_ENDPOINT"
    fi

    if ! $AWS_CMD "$s3_path" "$temp_backup"; then
        log_error "Failed to download from S3"
        rm -f "$temp_backup"
        exit 1
    fi

    # Restore from downloaded file
    restore_from_file "$temp_backup"

    # Cleanup
    rm -f "$temp_backup"
}

# Main
main() {
    echo "======================================"
    echo "PuppetMaster2 Database Restore"
    echo "======================================"
    echo ""

    # Parse arguments
    if [ $# -eq 0 ]; then
        # No arguments - list backups
        list_backups
        echo ""
        log_info "Usage:"
        echo "  $0 <backup_file>        - Restore specific backup"
        echo "  $0 --latest             - Restore latest backup"
        echo "  $0 --from-s3 <s3_path>  - Restore from S3"
        exit 0
    fi

    case "$1" in
        --latest)
            LATEST=$(get_latest_backup)
            if [ -z "$LATEST" ]; then
                log_error "No backups found"
                exit 1
            fi
            log_info "Latest backup: $LATEST"
            echo ""
            read -p "Are you sure you want to restore from this backup? [y/N] " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                restore_from_file "$LATEST"
            else
                log "Restore cancelled"
                exit 0
            fi
            ;;
        --from-s3)
            if [ -z "${2:-}" ]; then
                log_error "Please specify S3 path"
                exit 1
            fi
            echo ""
            read -p "Are you sure you want to restore from S3? [y/N] " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                restore_from_s3 "$2"
            else
                log "Restore cancelled"
                exit 0
            fi
            ;;
        --list|-l)
            list_backups
            ;;
        *)
            # Assume it's a backup file path
            if [ -f "$1" ]; then
                echo ""
                read -p "Are you sure you want to restore from $1? [y/N] " -n 1 -r
                echo
                if [[ $REPLY =~ ^[Yy]$ ]]; then
                    restore_from_file "$1"
                else
                    log "Restore cancelled"
                    exit 0
                fi
            else
                log_error "File not found: $1"
                exit 1
            fi
            ;;
    esac
}

main "$@"
