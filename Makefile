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
init: _init-check-tools _init-install-deps _init-build _init-validate ## Initialize workspace
	@$(call print_success,Workspace initialized successfully!)

.PHONY: _init-check-tools
_init-check-tools: ## [Phase 1] Check prerequisites
	@$(call print_header,📦 Phase 1/4 — Checking Prerequisites)
	@$(call print_substep,Running on: $(OS) ($(UNAME_S)))
	@for tool in rustc cargo node npm pnpm; do \
		if $(CHECK_CMD) $$tool >$(NULL_DEV) 2>&1; then \
			printf "  $(GREEN)✅$(RESET) %-10s %s\n" "$$tool" "$$($$tool --version 2>&1 | head -1)"; \
		else \
			printf "  $(YELLOW)⚠️$(RESET) %-10s not found (install manually)\n" "$$tool"; \
		fi \
	done

.PHONY: _init-install-deps
_init-install-deps: ## [Phase 2] Install project dependencies
	@$(call print_header,📦 Phase 2/4 — Installing Dependencies)
	@$(call print_substep,Installing Node.js dependencies...)
	@(cd projects/node/react-web && npx pnpm install 2>/dev/null) || true
	@(cd projects/node/desktop-tauri && npm install 2>/dev/null) || true

.PHONY: _init-build
_init-build: ## [Phase 3] Build workspace
	@$(call print_header,📦 Phase 3/4 — Building Workspace)
	@$(call print_substep,Building Rust workspace crates...)
	@cargo build
	@$(call print_substep,Building AI engine...)
	@cargo build --manifest-path .workspace/ai/rust/Cargo.toml 2>/dev/null || \
		printf "  $(YELLOW)⚠️$(RESET) AI engine skipped (run 'make _init-build-ai' manually)\n"

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
