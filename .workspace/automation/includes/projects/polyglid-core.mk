# ============================================================================
# polyglid-core - Project Makefile
# ============================================================================

PROJECT_NAME := polyglid-core
PROJECT_LANGUAGE := rust
PROJECT_DIR := rust/crates/polyglid-core
PROJECT_PATH := projects/rust/crates/polyglid-core

.PHONY: polyglid-core-build polyglid-core-test polyglid-core-clean

polyglid-core-build:
	@echo "  Building polyglid-core..."
	@cd $(PROJECT_PATH) && $(call get_build_command,$(PROJECT_LANGUAGE))

polyglid-core-test:
	@echo "  Testing polyglid-core..."
	@cd $(PROJECT_PATH) && $(call get_test_command,$(PROJECT_LANGUAGE))

polyglid-core-clean:
	@echo "  Cleaning polyglid-core..."
	@cd $(PROJECT_PATH) && $(call get_clean_command,$(PROJECT_LANGUAGE))

polyglid-core-dev:
	@echo "  Starting polyglid-core development server..."
	@cd $(PROJECT_PATH) && $(call get_dev_command,$(PROJECT_LANGUAGE))
