.PHONY: dev-up dev-down migrate build logs shell frontend-dev

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

frontend-dev:
	cd frontend && npm install && npm run dev
