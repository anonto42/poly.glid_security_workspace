# ============================================================================
# Language-Specific Rules
# ============================================================================

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
