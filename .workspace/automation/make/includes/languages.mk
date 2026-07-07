# ============================================================================
# Language-Specific Rules
# ============================================================================

# Node.js
.PHONY: node-build node-test node-clean

node-build:
	@echo "  🟢 Building Node.js projects..."
	@cd projects/node/frontend/react-web && npx pnpm run build
	@cd projects/node/frontend/desktop-tauri && npm run build

node-test:
	@echo "  🟢 Testing Node.js projects..."
	@cd projects/node/frontend/react-web && npx pnpm run test

node-clean:
	@echo "  🟢 Cleaning Node.js projects..."
	@rm -rf projects/node/frontend/react-web/node_modules projects/node/frontend/react-web/dist
	@rm -rf projects/node/frontend/desktop-tauri/node_modules projects/node/frontend/desktop-tauri/dist

# Rust
.PHONY: rust-build rust-test rust-clean

rust-build:
	@echo "  🦀 Building Rust projects..."
	@cd projects/rust && cargo build --release

rust-test:
	@echo "  🦀 Testing Rust projects..."
	@cd projects/rust && cargo test --workspace

rust-clean:
	@echo "  🦀 Cleaning Rust projects..."
	@cd projects/rust && cargo clean
