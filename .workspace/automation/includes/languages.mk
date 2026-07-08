# ============================================================================
# Language-Specific Rules
# ============================================================================

# Node.js
.PHONY: node-build node-test node-clean

NODE_PKG := $(shell command -v pnpm 2>/dev/null && echo "pnpm" || echo "npm")

node-build:
	@echo "  🟢 Building Node.js projects..."
	@cd projects/node/react-web && $(NODE_PKG) run build
	@cd projects/node/desktop-tauri && npm run build

node-test:
	@echo "  🟢 Testing Node.js projects..."
	@cd projects/node/react-web && $(NODE_PKG) run test

node-clean:
	@echo "  🟢 Cleaning Node.js projects..."
	@rm -rf projects/node/react-web/node_modules projects/node/react-web/dist
	@rm -rf projects/node/desktop-tauri/node_modules projects/node/desktop-tauri/dist

# Rust
.PHONY: rust-build rust-test rust-clean

rust-build:
	@echo "  🦀 Building Rust projects..."
	@cargo build --release

rust-test:
	@echo "  🦀 Testing Rust projects..."
	@cargo test --workspace

rust-clean:
	@echo "  🦀 Cleaning Rust projects..."
	@cargo clean
