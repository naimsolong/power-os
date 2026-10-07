.PHONY: dev dev-up dev-down dev-stop migrate build logs shell frontend-dev be-check

# Run backend + frontend locally (database, migrations, backend in background, frontend in foreground)
dev:
	@./scripts/dev.sh

# Stop local dev backend/frontend and database
dev-stop:
	-@pkill -f 'power-os-api' 2>/dev/null || true
	@docker compose down db 2>/dev/null || true

# Full Docker Compose stack (builds backend image)
dev-up:
	@test -f .env || (echo "Please copy .env.example to .env and adjust values." && exit 1)
	docker compose up --build -d

dev-down:
	docker compose down

migrate:
	docker compose exec app sqlx migrate run --source crates/migrations/migrations

build:
	docker compose build app

logs:
	docker compose logs -f app

shell:
	docker compose exec app bash

# Run only frontend dev server (assumes backend is already running)
frontend-dev:
	cd frontend && npm install && npm run dev

# Run only backend dev server (assumes database is already running)
be-dev:
	. "$HOME/.cargo/env" && \
	docker compose up db -d --wait && \
	sqlx migrate run --source crates/migrations/migrations --database-url "postgres://postgres:postgres@localhost:5432/poweros" && \
	DATABASE_URL="postgres://postgres:postgres@localhost:5432/poweros" cargo run -p power-os-api

# Type-check backend
be-check:
	. "$HOME/.cargo/env" && cargo check
