#!/usr/bin/env bash
set -euo pipefail

# Power OS local development runner
# Starts PostgreSQL, runs migrations, starts backend (bg) and frontend (fg).
# Press Ctrl-C to stop frontend, backend, and database.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

if [[ ! -f .env ]]; then
  echo "Please copy .env.example to .env and adjust values first."
  exit 1
fi

# Load .env (ignore comments and empty lines)
set -a
# shellcheck source=/dev/null
source .env
set +a

export DATABASE_URL="${DATABASE_URL:-postgres://${POSTGRES_USER:-postgres}:${POSTGRES_PASSWORD:-postgres}@localhost:5432/${POSTGRES_DB:-poweros}}"

cleanup() {
  echo ""
  echo "Shutting down..."
  if [[ -n "${BACKEND_PID:-}" ]] && kill -0 "$BACKEND_PID" 2>/dev/null; then
    echo "Stopping backend (PID $BACKEND_PID)..."
    kill "$BACKEND_PID" 2>/dev/null || true
    wait "$BACKEND_PID" 2>/dev/null || true
  fi
  echo "Stopping database..."
  docker compose down db 2>/dev/null || true
  echo "Done."
}
trap cleanup EXIT INT TERM

echo "Starting PostgreSQL..."
docker compose up db -d --wait

echo "Running migrations..."
. "$HOME/.cargo/env"
sqlx migrate run --source crates/migrations/migrations

echo "Starting backend..."
cargo run -p power-os-api > /tmp/power-os-api.log 2>&1 &
BACKEND_PID=$!
sleep 3

if ! kill -0 "$BACKEND_PID" 2>/dev/null; then
  echo "Backend failed to start. Log:"
  cat /tmp/power-os-api.log
  exit 1
fi

echo "Backend running on http://localhost:3000 (PID $BACKEND_PID)"

echo "Starting frontend..."
cd frontend
npm install
npm run dev
