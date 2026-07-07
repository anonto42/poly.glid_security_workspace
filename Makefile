# ============================================================================
# PolyGlid Workspace Makefile
# ============================================================================
# Usage: make [target] [options]
# Examples:
#   make help           - Show all available commands
#   make dev            - Start all development servers
#   make build          - Build all projects
#   make test           - Run all tests
#   make clean          - Clean all artifacts
# ============================================================================

.ONESHELL:
.DELETE_ON_ERROR:
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

# Include OS detection (first — other includes depend on these vars)
include .workspace/automation/includes/os.mk

# Include automation system
include .workspace/automation/includes/colors.mk
include .workspace/automation/includes/config.mk
include .workspace/automation/includes/utils.mk
include .workspace/automation/includes/help.mk

# Include language modules
include .workspace/automation/includes/languages.mk

# Include infrastructure modules
include .workspace/automation/includes/docker.mk
include .workspace/automation/includes/k8s.mk
include .workspace/automation/includes/ci.mk

# ============================================================================
# Default Target
# ============================================================================

.DEFAULT_GOAL := help

# ============================================================================
# Workspace Commands
# ============================================================================

# ── Init (orchestrator — runs phases in order) ──

.PHONY: init
init: _init-check-tools _init-install-deps _init-build _init-validate ## Initialize workspace (check, install, build, validate)
	@$(call print_success,Workspace initialized successfully!)

.PHONY: _init-check-tools
_init-check-tools: _check-dev-tools _check-git _check-docker _check-ollama _check-system _check-gpu _setup-missing-tools _setup-ollama-model _setup-ai-config ## [Phase 1] Check prerequisites & auto-setup
	@$(call print_success,All prerequisite checks & auto-setup complete!)

# ── Tool checks ──

.PHONY: _check-dev-tools
_check-dev-tools:
	@$(call print_header,📦 Phase 1/6 — Development Tools)
	@$(call print_substep,Running on: $(OS) ($(UNAME_S)) ($(UNAME_M)))
	@for tool in rustc cargo rustup node npm pnpm; do \
		if $(CHECK_CMD) $$tool >$(NULL_DEV) 2>&1; then \
			printf "  $(GREEN)✅$(RESET) %-12s %s\n" "$$tool" "$$($$tool --version 2>&1 | head -1)"; \
		else \
			printf "  $(YELLOW)⚠️$(RESET) %-12s not found (install manually)\n" "$$tool"; \
		fi \
	done

.PHONY: _check-git
_check-git:
	@$(call print_header,📦 Phase 2/6 — Git Configuration)
	@if $(CHECK_CMD) git >$(NULL_DEV) 2>&1; then \
		name=$$(git config --global user.name 2>/dev/null || echo ""); \
		email=$$(git config --global user.email 2>/dev/null || echo ""); \
		printf "  $(GREEN)✅$(RESET) %-12s %s\n" "git" "$$(git --version 2>&1 | head -1)"; \
		if [ -n "$$name" ]; then \
			printf "  $(GREEN)✅$(RESET) %-12s %s\n" "user.name" "$$name"; \
		else \
			printf "  $(YELLOW)⚠️$(RESET) %-12s not set (run: git config --global user.name \"Your Name\")\n" "user.name"; \
		fi; \
		if [ -n "$$email" ]; then \
			printf "  $(GREEN)✅$(RESET) %-12s %s\n" "user.email" "$$email"; \
		else \
			printf "  $(YELLOW)⚠️$(RESET) %-12s not set (run: git config --global user.email \"you@example.com\")\n" "user.email"; \
		fi; \
	else \
		printf "  $(YELLOW)⚠️$(RESET) %-12s not found\n" "git"; \
	fi

.PHONY: _check-docker
_check-docker:
	@$(call print_header,📦 Phase 3/6 — Docker & Containers)
	@if $(CHECK_CMD) docker >$(NULL_DEV) 2>&1; then \
		printf "  $(GREEN)✅$(RESET) %-12s %s\n" "docker" "$$(docker --version 2>&1)"; \
		if docker info >$(NULL_DEV) 2>&1; then \
			printf "  $(GREEN)✅$(RESET) %-12s daemon running\n" "docker"; \
		else \
			printf "  $(YELLOW)⚠️$(RESET) %-12s daemon not running (start Docker Desktop or service)\n" "docker"; \
		fi; \
		if docker compose version >$(NULL_DEV) 2>&1; then \
			printf "  $(GREEN)✅$(RESET) %-12s %s\n" "compose" "$$(docker compose version 2>&1)"; \
		else \
			printf "  $(YELLOW)⚠️$(RESET) %-12s not available\n" "compose"; \
		fi; \
	else \
		printf "  $(YELLOW)⚠️$(RESET) %-12s not found (install Docker)\n" "docker"; \
	fi

.PHONY: _check-ollama
_check-ollama:
	@$(call print_header,📦 Phase 4/6 — Ollama AI)
	@if $(CHECK_CMD) ollama >$(NULL_DEV) 2>&1; then \
		ver=$$(ollama --version 2>&1 | grep -oP 'version is \K\S+' || echo "installed"); \
		printf "  $(GREEN)✅$(RESET) %-12s %s\n" "ollama" "$$ver"; \
		models=$$(ollama list 2>&1 | tail -n +2 | awk '{print $$1}' | tr '\n' ' ' || true); \
		if [ -n "$$models" ]; then \
			printf "  $(GREEN)✅$(RESET) %-12s %s\n" "models" "$$models"; \
		else \
			if ollama list >$(NULL_DEV) 2>&1; then \
				printf "  $(YELLOW)ℹ️$(RESET)  %-12s no models pulled yet\n" "models"; \
			else \
				printf "  $(YELLOW)⚠️$(RESET) %-12s daemon not running (start with: ollama serve)\n" "daemon"; \
			fi; \
		fi; \
	else \
		printf "  $(YELLOW)⚠️$(RESET) %-12s not found (install from ollama.com)\n" "ollama"; \
	fi

.PHONY: _check-system
_check-system:
	@$(call print_header,📦 Phase 5/6 — System Resources)
	@printf "  $(GREEN)✅$(RESET) %-12s %s\n" "CPU" "$$(nproc 2>/dev/null || echo unknown) cores"
	@memory=$$(free -h 2>/dev/null | awk '/^Mem:/{print $$2}'); \
	if [ -n "$$memory" ]; then \
		printf "  $(GREEN)✅$(RESET) %-12s %s\n" "RAM" "$$memory"; \
	fi
	@disk=$$(df -h . 2>/dev/null | awk 'NR==2{print $$4}'); \
	if [ -n "$$disk" ]; then \
		printf "  $(GREEN)✅$(RESET) %-12s %s free\n" "disk" "$$disk"; \
	fi
	@printf "  $(GREEN)✅$(RESET) %-12s %s\n" "shell" "$$($(SHELL) --version 2>&1 | head -1)"

.PHONY: _check-gpu
_check-gpu:
	@$(call print_header,📦 Phase 6/6 — GPU Detection)
	@if $(CHECK_CMD) nvidia-smi >$(NULL_DEV) 2>&1; then \
		gpu=$$(nvidia-smi --query-gpu=name,memory.total --format=csv,noheader 2>/dev/null | head -1); \
		printf "  $(GREEN)✅$(RESET) %-12s %s\n" "NVIDIA" "$$gpu"; \
		printf "  $(GREEN)💡$(RESET) %-12s Use: ollama pull codellama:13b (GPU recommended)\n" "model"; \
	else \
		printf "  $(YELLOW)ℹ️$(RESET)  %-12s no NVIDIA GPU detected\n" "gpu"; \
		if [ "$$(uname -m)" = "arm64" ] || [ "$$(uname -m)" = "aarch64" ]; then \
			printf "  $(GREEN)💡$(RESET) %-12s Apple Silicon detected — ollama with Metal works\n" "note"; \
			printf "  $(GREEN)💡$(RESET) %-12s Use: ollama pull codellama:7b\n" "model"; \
		else \
			printf "  $(GREEN)💡$(RESET) %-12s CPU-only — use small models: phi3:3.8b\n" "model"; \
		fi; \
	fi

# ── Auto-setup (installs missing tools when possible) ──

.PHONY: _setup-missing-tools
_setup-missing-tools:
	@$(call print_header,🔧 Auto-Setup — Installing Missing Tools)
	@installed=0; \
	\
	## Rust via rustup \
	if ! $(CHECK_CMD) rustc >$(NULL_DEV) 2>&1; then \
		printf "  $(YELLOW)⏳$(RESET) Installing Rust (rustup)...\n"; \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
			| sh -s -- -y 2>&1 | tail -1 || true; \
		. $$HOME/.cargo/env 2>/dev/null || true; \
		_rustc_ver=$$(rustc --version 2>&1) || true; \
		if [ -n "$$_rustc_ver" ]; then \
			printf "  $(GREEN)✅$(RESET) %-12s %s\n" "rustc" "$$_rustc_ver"; \
			installed=1; \
		else \
			printf "  $(YELLOW)⚠️$(RESET) %-12s install failed\n" "rustc"; \
		fi; \
	else \
		printf "  $(GREEN)✅$(RESET) %-12s already installed\n" "rustc"; \
	fi; \
	\
	## pnpm via npm \
	if ! $(CHECK_CMD) pnpm >$(NULL_DEV) 2>&1; then \
		if $(CHECK_CMD) npm >$(NULL_DEV) 2>&1; then \
			printf "  $(YELLOW)⏳$(RESET) Installing pnpm via npm...\n"; \
			_npm_prefix=$$(npm config get prefix); \
			if [ "$$_npm_prefix" = "/usr" ]; then \
				mkdir -p "$$HOME/.npm-global"; \
				npm config set prefix "$$HOME/.npm-global" 2>/dev/null; \
			fi; \
			npm install -g pnpm 2>/dev/null || true; \
			_pnpm_bin="$$(npm config get prefix)/bin/pnpm"; \
			if [ -x "$$_pnpm_bin" ]; then \
				_pnpm_ver=$$("$$_pnpm_bin" --version 2>&1) || true; \
				printf "  $(GREEN)✅$(RESET) %-12s %s\n" "pnpm" "$$_pnpm_ver"; \
				installed=1; \
			else \
				printf "  $(YELLOW)⚠️$(RESET) %-12s install failed (try: sudo npm install -g pnpm)\n" "pnpm"; \
			fi; \
		else \
			printf "  $(YELLOW)⚠️$(RESET) %-12s npm missing — install pnpm manually\n" "pnpm"; \
		fi; \
	else \
		printf "  $(GREEN)✅$(RESET) %-12s already installed\n" "pnpm"; \
	fi; \
	\
	## Ollama \
	if ! $(CHECK_CMD) ollama >$(NULL_DEV) 2>&1; then \
		if [ "$(UNAME_S)" = "Linux" ] || [ "$(UNAME_S)" = "Darwin" ]; then \
			printf "  $(YELLOW)⏳$(RESET) Installing Ollama...\n"; \
			curl -fsSL https://ollama.com/install.sh | sh 2>&1 | tail -3 || true; \
			if $(CHECK_CMD) ollama >$(NULL_DEV) 2>&1; then \
				printf "  $(GREEN)✅$(RESET) %-12s %s\n" "ollama" "$$(ollama --version 2>&1)"; \
				installed=1; \
			else \
				printf "  $(YELLOW)⚠️$(RESET) %-12s install failed — see ollama.com\n" "ollama"; \
			fi; \
		else \
			printf "  $(YELLOW)⚠️$(RESET) %-12s install manually from ollama.com\n" "ollama"; \
		fi; \
	else \
		printf "  $(GREEN)✅$(RESET) %-12s already installed\n" "ollama"; \
	fi; \
	\
	## Git user config \
	if $(CHECK_CMD) git >$(NULL_DEV) 2>&1; then \
		name=$$(git config --global user.name 2>/dev/null || echo ""); \
		email=$$(git config --global user.email 2>/dev/null || echo ""); \
		if [ -z "$$name" ]; then \
			printf "  $(YELLOW)⚠️$(RESET) %-12s run: git config --global user.name \"Your Name\"\n" "user.name"; \
		fi; \
		if [ -z "$$email" ]; then \
			printf "  $(YELLOW)⚠️$(RESET) %-12s run: git config --global user.email \"you@example.com\"\n" "user.email"; \
		fi; \
	fi; \
	\
	if [ "$$installed" -eq 0 ]; then \
		printf "  $(GREEN)✅$(RESET) All tools already present — nothing to install\n"; \
	fi

.PHONY: _setup-ollama-model
_setup-ollama-model:
	@if $(CHECK_CMD) ollama >$(NULL_DEV) 2>&1; then \
		if ollama list >$(NULL_DEV) 2>&1; then \
			models=$$(ollama list 2>/dev/null | tail -n +2 | wc -l); \
			if [ "$$models" -eq 0 ]; then \
				$(call print_header,🤖 Auto-Setup — Pulling Recommended Ollama Model); \
				if $(CHECK_CMD) nvidia-smi >$(NULL_DEV) 2>&1; then \
					model="codellama:7b"; \
				elif [ "$$(uname -m)" = "arm64" ] || [ "$$(uname -m)" = "aarch64" ]; then \
					model="codellama:7b"; \
				else \
					model="phi3:3.8b"; \
				fi; \
				printf "  $(YELLOW)⏳$(RESET) Pulling %s (this may take a while)...\n" "$$model"; \
				ollama pull "$$model" 2>&1 | tail -1; \
				printf "  $(GREEN)✅$(RESET) Model %s ready\n" "$$model"; \
			fi; \
		fi; \
	fi

.PHONY: _setup-ai-config
_setup-ai-config:
	@$(call print_header,🔧 Auto-Setup — Generating AI Configuration)
	@config_dir=".workspace/ai/configs"; \
	config_file="$$config_dir/ai-config.toml"; \
	mkdir -p "$$config_dir"; \
	\
	## Detect recommended model \
	if $(CHECK_CMD) nvidia-smi >$(NULL_DEV) 2>&1; then \
		recommended="codellama:7b"; \
	elif [ "$$(uname -m)" = "arm64" ] || [ "$$(uname -m)" = "aarch64" ]; then \
		recommended="codellama:7b"; \
	else \
		recommended="phi3:3.8b"; \
	fi; \
	\
	## Check if Ollama has any models pulled \
	if $(CHECK_CMD) ollama >$(NULL_DEV) 2>&1; then \
		if ollama list >$(NULL_DEV) 2>&1; then \
			avail=$$(ollama list 2>/dev/null | tail -n +2 | head -1 | awk '{print $$1}'); \
			if [ -n "$$avail" ]; then \
				recommended="$$avail"; \
			fi; \
		fi; \
	fi; \
	\
	## Write config (only if changed or missing) \
	if [ -f "$$config_file" ]; then \
		current_model=$$(grep -E '^model\s*=' "$$config_file" | head -1 | sed 's/.*= *"\(.*\)"/\1/'); \
	else \
		current_model=""; \
	fi; \
	if [ "$$current_model" != "$$recommended" ]; then \
		printf "  $(YELLOW)⏳$(RESET) Writing ai-config.toml (model: %s)...\n" "$$recommended"; \
		{ \
			echo 'provider_type = "Ollama"'; \
			echo 'api_base = "http://localhost:11434/v1"'; \
			echo 'api_key = ""'; \
			echo "model = \"$$recommended\""; \
			echo 'temperature = 0.7'; \
			echo 'max_tokens = 4096'; \
			echo 'cache_enabled = true'; \
			echo 'auto_suggestions = true'; \
			echo 'suggestion_interval = 3600'; \
			echo ''; \
			echo '[models]'; \
			echo "code     = \"$$recommended\""; \
			echo "security = \"$$recommended\""; \
			echo "build    = \"$$recommended\""; \
			echo "suggest  = \"$$recommended\""; \
		} > "$$config_file"; \
		printf "  $(GREEN)✅$(RESET) %-12s %s\n" "config" "$$recommended"; \
	else \
		printf "  $(GREEN)✅$(RESET) %-12s %s (unchanged)\n" "config" "$$recommended"; \
	fi

# ── Init Phase 2: Install Dependencies ──

.PHONY: _init-install-deps
_init-install-deps: ## [Phase 2] Install project dependencies
	@$(call print_header,📦 Phase 2/4 — Installing Dependencies)
	@$(call print_substep,Installing Node.js dependencies...)
	@(cd projects/node/react-web && npx pnpm install 2>/dev/null) || true
	@(cd projects/node/desktop-tauri && npm install 2>/dev/null) || true

.PHONY: _init-build
_init-build: build-rust build-ai-engine ## [Phase 3] Build workspace

.PHONY: _init-validate
_init-validate: ## [Phase 4] Validate workspace structure
	@$(call print_header,📦 Phase 4/4 — Validating Workspace)
	@.workspace/automation/scripts/validate-workspace.sh

.PHONY: status
status: ## Show workspace status
	@$(call print_header,📊 Workspace Status)
	@echo "  $(GREEN)✓$(RESET) Workspace root: $(WORKSPACE_ROOT)"
	@echo "  $(GREEN)✓$(RESET) Languages enabled: $(LANGUAGES)"
	@$(call print_step,Project Health:)
	@.workspace/automation/scripts/validate-workspace.sh --quiet

.PHONY: graph
graph: ## Generate and display dependency graph
	@$(call print_header,📊 Dependency Graph)
	@.workspace/automation/scripts/generate-graph.sh

.PHONY: info
info: ## Show workspace information
	@$(call print_header,📋 Workspace Information)
	@echo "  Name: $(WORKSPACE_NAME)"
	@echo "  Version: $(WORKSPACE_VERSION)"
	@echo "  Root: $(WORKSPACE_ROOT)"
	@echo "  Languages: $(LANGUAGES)"
	@echo "  Make version: $(MAKE_VERSION)"
	@echo "  Shell: $(SHELL)"

# ============================================================================
# Development Commands
# ============================================================================

.PHONY: dev
dev: ## Start development servers (usage: make dev)
	@$(call print_header,🚀 Starting Development Servers)
	@$(MAKE) -j$(PARALLEL_JOBS) dev-node dev-rust

.PHONY: dev-node
dev-node:
	@$(call print_substep,Starting Node/TypeScript dev servers...)
	@(cd projects/node/react-web && npx pnpm run dev) &
	@(cd projects/node/desktop-tauri && npm run dev) &

.PHONY: dev-rust
dev-rust:
	@$(call print_substep,Starting Rust backend server...)
	@cargo run -p polyglid-server &

# ============================================================================
# Build Commands
# ============================================================================

.PHONY: build
build: ## Build all projects
	@$(call print_header,📦 Building All Projects)
	@$(MAKE) build-rust build-node

.PHONY: build-node
build-node:
	@$(call print_substep,Building Node/TypeScript projects...)
	@(cd projects/node/react-web && npx pnpm run build)
	@(cd projects/node/desktop-tauri && npm run build)

.PHONY: build-rust
build-rust:
	@$(call print_substep,Building Rust workspace crates...)
	@cargo build --release

# ============================================================================
# Test Commands
# ============================================================================

.PHONY: test
test: ## Run all tests
	@$(call print_header,🧪 Running All Tests)
	@$(MAKE) test-rust test-node

.PHONY: test-rust
test-rust:
	@$(call print_substep,Testing Rust workspace...)
	@cargo test --all

# ============================================================================
# Clean Commands
# ============================================================================

.PHONY: clean
clean: ## Clean all build artifacts
	@$(call print_header,🧹 Cleaning Workspace)
	@$(MAKE) clean-node clean-rust
	@$(call print_success,Clean completed!)

.PHONY: clean-node
clean-node:
	@$(call print_substep,Cleaning Node.js projects...)
	@rm -rf projects/node/react-web/node_modules projects/node/react-web/dist
	@rm -rf projects/node/desktop-tauri/node_modules projects/node/desktop-tauri/dist

.PHONY: clean-rust
clean-rust:
	@$(call print_substep,Cleaning Rust projects...)
	@cargo clean

# ============================================================================
# AI Commands
# ============================================================================

AI_BIN := .workspace/ai/rust/target/release/polyglid-ai
ARGS ?=
QUERY ?=

.PHONY: build-ai-engine
build-ai-engine: ## Build AI engine binary (release)
	@$(call print_substep,Building AI engine (release)...)
	@cargo build --release --manifest-path .workspace/ai/rust/Cargo.toml 2>&1 | tail -3 || \
		printf "  $(YELLOW)⚠️$(RESET) AI engine build failed\n"

.PHONY: ai-analyze
ai-analyze: build-ai-engine ## Run AI workspace analysis
	@$(call print_header,🤖 AI Workspace Analysis)
	@if [ -f $(AI_BIN) ]; then \
		$(AI_BIN) analyze $$(ARGS); \
	else \
		printf "  $(YELLOW)⚠️$(RESET) AI engine binary not found\n"; \
	fi

.PHONY: ai-suggest
ai-suggest: build-ai-engine ## Get AI suggestions
	@$(call print_header,💡 AI Suggestions)
	@if [ -f $(AI_BIN) ]; then \
		$(AI_BIN) suggest --limit 10 $(ARGS); \
	else \
		printf "  $(YELLOW)⚠️$(RESET) AI engine binary not found\n"; \
	fi

.PHONY: ai-security
ai-security: build-ai-engine ## Run AI security scan
	@$(call print_header,🔒 AI Security Scan)
	@if [ -f $(AI_BIN) ]; then \
		$(AI_BIN) security $(ARGS); \
	else \
		printf "  $(YELLOW)⚠️$(RESET) AI engine binary not found\n"; \
	fi

.PHONY: ai-status
ai-status: build-ai-engine ## Show AI engine status
	@$(call print_header,📊 AI Engine Status)
	@if [ -f $(AI_BIN) ]; then \
		$(AI_BIN) status $(ARGS); \
	else \
		printf "  $(YELLOW)⚠️$(RESET) AI engine binary not found\n"; \
	fi

.PHONY: ai-ingest
ai-ingest: build-ai-engine ## Build code vector index
	@$(call print_header,📥 Ingest Workspace Code)
	@if [ -f $(AI_BIN) ]; then \
		$(AI_BIN) ingest $(ARGS); \
	else \
		printf "  $(YELLOW)⚠️$(RESET) AI engine binary not found\n"; \
	fi

.PHONY: ai-search
ai-search: build-ai-engine ## Search code index (usage: make ai-search QUERY="find auth logic")
	@$(call print_header,🔍 Search Code Index)
	@if [ -f $(AI_BIN) ]; then \
		$(AI_BIN) search "$(QUERY)" $(ARGS); \
	else \
		printf "  $(YELLOW)⚠️$(RESET) AI engine binary not found\n"; \
	fi

# ============================================================================
# Deploy Command (stub — real impl depends on Docker + K8s being ready)
# ============================================================================

.PHONY: deploy
deploy: build _deploy-docker _deploy-k8s ## Build and deploy (stub)
	@$(call print_success,Deploy complete!)

.PHONY: _deploy-docker
_deploy-docker:
	@$(call print_substep,Building Docker images...)
	@$(MAKE) docker-up

.PHONY: _deploy-k8s
_deploy-k8s:
	@$(call print_substep,Deploying to Kubernetes...)
	@$(MAKE) k8s-apply

# ============================================================================
# New Project Scaffolding
# ============================================================================

.PHONY: new-project
new-project: ## Create a new project from template
	@$(call print_header,📁 New Project)
	@read -p "  Language (rust/node/python/go): " lang; \
	read -p "  Project name: " name; \
	template=".workspace/automation/templates/project.mk.template"; \
	path="projects/$$lang/$$name"; \
	mkdir -p "$$path"; \
	printf "  $(GREEN)✅$(RESET) Created $$path\n"; \
	printf "  $(GREEN)💡$(RESET) Add '$$name = { path = \"$$path\", language = \"$$lang\", type = \"service\" }' to workspace.toml\n"
