# ============================================================================
# polyglid-desktop - Project Makefile
# ============================================================================

PROJECT_NAME := polyglid-desktop
PROJECT_LANGUAGE := rust
PROJECT_DIR := polyglid-desktop
PROJECT_PATH := slices/apps/desktop

.PHONY: polyglid-desktop-build polyglid-desktop-test polyglid-desktop-clean polyglid-desktop-dev

polyglid-desktop-build:
	@echo "  Building polyglid-desktop..."
	@cargo build -p polyglid-desktop

polyglid-desktop-test:
	@echo "  Testing polyglid-desktop..."
	@cargo test -p polyglid-desktop

polyglid-desktop-clean:
	@echo "  Cleaning polyglid-desktop..."
	@cargo clean -p polyglid-desktop

polyglid-desktop-dev:
	@echo "  Starting polyglid-desktop..."
	@cargo run -p polyglid-desktop
