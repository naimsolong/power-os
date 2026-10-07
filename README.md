# Power OS

Open-source, self-hostable business operating system for Malaysian SMEs.

## Quick start (self-hosted)

This repository is designed to run on a single VPS using Docker Compose.

### Prerequisites

- [Docker](https://docs.docker.com/engine/install/)
- [Docker Compose](https://docs.docker.com/compose/install/)
- [Make](https://www.gnu.org/software/make/) (usually pre-installed on macOS/Linux)
- Git

### 1. Clone the repository

```bash
git clone https://github.com/power-os/power-os.git
cd power-os
```

### 2. Configure the environment

```bash
cp .env.example .env
```

Review `.env` and change any defaults (especially `POSTGRES_PASSWORD` and `MINIO_ROOT_PASSWORD`) before running in production.

### 3. Start the backend and services

```bash
make dev-up
```

This builds the Rust backend and starts PostgreSQL 16, MinIO, and Caddy in the background. The backend is available at:

- Direct: http://localhost:3000
- Through Caddy: http://localhost (on a clean VPS; port 80 may be occupied by other local tools such as Laravel Herd on a dev Mac)

MinIO ports are mapped to `9002` (API) and `9003` (console) on the host to avoid common conflicts.

Run migrations once on first setup (and after any migration is added):

```bash
make migrate
```

### 4. Verify

```bash
curl http://localhost:3000/health
```

You should see `ok`.

### 5. Start the frontend (separately)

The frontend is a Vite + React + TypeScript SPA and is started outside of Docker for the best dev experience:

```bash
cd frontend
npm install
npm run dev
```

The frontend dev server usually runs on http://localhost:5173.

## Local development (quickest)

To run both backend and frontend together locally:

```bash
make dev
```

This command will:
1. Start PostgreSQL in Docker
2. Run migrations
3. Start the Rust backend in the background
4. Start the Vite frontend dev server in the foreground

Press `Ctrl-C` to stop the frontend, backend, and database.

You can also run them separately:

| Command | Description |
|---------|-------------|
| `make be-dev` | Start database + migrations + backend |
| `make frontend-dev` | Start frontend dev server only |
| `make be-check` | Type-check the backend |
| `make dev-stop` | Stop background backend and database |

## Useful commands

| Command | Description |
|---------|-------------|
| `make dev` | Start local backend + frontend |
| `make dev-up` | Build and start all Docker services |
| `make dev-down` | Stop all Docker services |
| `make dev-stop` | Stop local backend/database |
| `make migrate` | Run database migrations |
| `make build` | Build the app image only |
| `make logs` | Tail backend logs |
| `make shell` | Open a shell inside the app container |

## Services

| Service | Image | Purpose |
|---------|-------|---------|
| app | `Dockerfile` (Rust) | Axum backend API |
| db | `postgres:16-alpine` | Primary database |
| minio | `quay.io/minio/minio` | Object storage for uploads (host ports 9002/9003) |
| caddy | `caddy:2-alpine` | Reverse proxy |

## Backend development (without Docker)

If you prefer to run the Rust backend directly for faster iteration:

### Requirements

- Rust toolchain (latest stable)
- PostgreSQL 16 (via Docker or local install)
- `sqlx-cli`

```bash
# Install sqlx-cli
cargo install sqlx-cli --features native-tls,postgres --no-default-features

# Start PostgreSQL
docker run -d --name power-os-postgres \
  -e POSTGRES_USER=poweros \
  -e POSTGRES_PASSWORD=poweros \
  -e POSTGRES_DB=poweros \
  -p 5432:5432 \
  postgres:16

# Run migrations
export DATABASE_URL="postgres://poweros:poweros@localhost:5432/poweros"
sqlx migrate run --source crates/migrations/migrations

# Run API
cargo run -p power-os-api
```

### Project layout

- `crates/api` — Axum HTTP server and routes.
- `crates/migrations` — SQLx migrations and migration runner.
- `crates/domain` — Shared domain types and identifiers.

## Frontend development

```bash
cd frontend
npm install
npm run dev
```

The dev server runs at http://localhost:5173.

## Production notes

- Change all default passwords in `.env`.
- Use a proper domain in `Caddyfile` and let Caddy provision TLS automatically.
- Back up the `postgres-data` and `minio-data` Docker volumes regularly.
- Build the smaller production image:
  ```bash
  docker compose build --target production app
  ```

## License

MIT
