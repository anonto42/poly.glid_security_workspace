# ============================================================================
# Language-Specific Rules
# ============================================================================

# Node.js
.PHONY: node-build node-test node-clean

NODE_PKG := $(shell command -v pnpm 2>/dev/null && echo "pnpm" || echo "npm")

node-build:
	@echo "  🟢 Building Node.js projects..."
	@cd projects/polyglid-web-legacy && $(NODE_PKG) run build
	@cd projects/polyglid-desktop-legacy && npm run build

node-test:
	@echo "  🟢 Testing Node.js projects..."
	@cd projects/polyglid-web-legacy && $(NODE_PKG) run test

node-clean:
	@echo "  🟢 Cleaning Node.js projects..."
	@rm -rf projects/polyglid-web-legacy/node_modules projects/polyglid-web-legacy/dist
	@rm -rf projects/polyglid-desktop-legacy/node_modules projects/polyglid-desktop-legacy/dist

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
