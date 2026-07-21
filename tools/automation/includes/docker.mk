# ============================================================================
# Docker Commands
# ============================================================================

DOCKER_COMPOSE := .workspace/infrastructure/docker-compose.yml
DOCKER_COMPOSE_ALT := infrastructure/docker-compose.yml

.PHONY: docker-up docker-down docker-build

docker-up:
	@echo "🐳 Starting local services via docker-compose..."
	@if [ -f $(DOCKER_COMPOSE) ]; then \
		docker compose -f $(DOCKER_COMPOSE) up -d 2>/dev/null; \
	elif [ -f $(DOCKER_COMPOSE_ALT) ]; then \
		docker compose -f $(DOCKER_COMPOSE_ALT) up -d 2>/dev/null; \
	else \
		docker compose up -d 2>/dev/null || \
		echo "  ⚠️  No docker-compose.yml found — start services manually"; \
	fi

docker-down:
	@echo "🐳 Stopping local services..."
	@if [ -f $(DOCKER_COMPOSE) ]; then \
		docker compose -f $(DOCKER_COMPOSE) down 2>/dev/null; \
	elif [ -f $(DOCKER_COMPOSE_ALT) ]; then \
		docker compose -f $(DOCKER_COMPOSE_ALT) down 2>/dev/null; \
	else \
		docker compose down 2>/dev/null || true; \
	fi

docker-build:
	@echo "🐳 Building Docker images..."
	@docker compose build 2>/dev/null || echo "  ⚠️  Docker build failed — is Docker running?"
