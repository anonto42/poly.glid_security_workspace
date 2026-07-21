# ============================================================================
# polyglid-plugin-api - Project Makefile
# ============================================================================

PROJECT_NAME := polyglid-plugin-api
PROJECT_LANGUAGE := rust
PROJECT_DIR := polyglid-plugin-api
PROJECT_PATH := crates/plugin-api

.PHONY: polyglid-plugin-api-build polyglid-plugin-api-test polyglid-plugin-api-clean

polyglid-plugin-api-build:
	@echo "  Building polyglid-plugin-api..."
	@cd $(PROJECT_PATH) && $(call get_build_command,$(PROJECT_LANGUAGE))

polyglid-plugin-api-test:
	@echo "  Testing polyglid-plugin-api..."
	@cd $(PROJECT_PATH) && $(call get_test_command,$(PROJECT_LANGUAGE))

polyglid-plugin-api-clean:
	@echo "  Cleaning polyglid-plugin-api..."
	@cd $(PROJECT_PATH) && $(call get_clean_command,$(PROJECT_LANGUAGE))

polyglid-plugin-api-dev:
	@echo "  Starting polyglid-plugin-api development server..."
	@cd $(PROJECT_PATH) && $(call get_dev_command,$(PROJECT_LANGUAGE))
