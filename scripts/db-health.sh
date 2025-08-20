#!/usr/bin/env bash
# Simple PostgreSQL health check / wait script.
#
# Usage:
#   scripts/db-health.sh check   # exit 0 if healthy, else 1
#   scripts/db-health.sh wait    # wait until healthy (with timeout)
#
# Environment variables (with defaults):
#   PGHOST (localhost)
#   PGPORT (5432)
#   PGUSER (capsule)
#   PGPASSWORD (capsule_password)  # not exported here; export in your shell if needed
#   PGDATABASE (capsule_dev)
#   DB_WAIT_TIMEOUT (30) seconds total
#   DB_WAIT_INTERVAL (2) seconds between retries
#
# The script prefers `pg_isready` if present; falls back to a simple `psql` query.
# Works for local docker-compose setup.
set -euo pipefail

MODE=${1:-check}

PGHOST=${PGHOST:-localhost}
PGPORT=${PGPORT:-5432}
PGUSER=${PGUSER:-capsule}
PGDATABASE=${PGDATABASE:-capsule_dev}
# PGPASSWORD should be set in environment if needed; don't hard-code secrets.
DB_WAIT_TIMEOUT=${DB_WAIT_TIMEOUT:-30}
DB_WAIT_INTERVAL=${DB_WAIT_INTERVAL:-2}

log() { printf '[db-health] %s\n' "$*"; }

has_pg_isready() {
  command -v pg_isready >/dev/null 2>&1
}

check_once() {
  if has_pg_isready; then
    if pg_isready -h "$PGHOST" -p "$PGPORT" -d "$PGDATABASE" -U "$PGUSER" >/dev/null 2>&1; then
      return 0
    else
      return 1
    fi
  else
    # Fallback: attempt a trivial query
    if PGPASSWORD="$PGPASSWORD" psql "postgresql://$PGUSER@$PGHOST:$PGPORT/$PGDATABASE" -c 'SELECT 1;' >/dev/null 2>&1; then
      return 0
    else
      return 1
    fi
  fi
}

case "$MODE" in
  check)
    if check_once; then
      log "database healthy ($PGHOST:$PGPORT/$PGDATABASE)"
      exit 0
    else
      log "database NOT healthy ($PGHOST:$PGPORT/$PGDATABASE)"
      exit 1
    fi
    ;;
  wait)
    log "Waiting for database at $PGHOST:$PGPORT/$PGDATABASE (timeout=${DB_WAIT_TIMEOUT}s)"
    start_ts=$(date +%s)
    while true; do
      if check_once; then
        log "database ready"
        exit 0
      fi
      now=$(date +%s)
      elapsed=$(( now - start_ts ))
      if [ "$elapsed" -ge "$DB_WAIT_TIMEOUT" ]; then
        log "timeout after ${elapsed}s waiting for database"
        exit 1
      fi
      sleep "$DB_WAIT_INTERVAL"
    done
    ;;
  *)
    echo "Unknown mode: $MODE" >&2
    echo "Usage: $0 [check|wait]" >&2
    exit 2
    ;;
esac
