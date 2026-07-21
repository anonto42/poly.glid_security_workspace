# ============================================================================
# polyglid-server - Project Makefile
# ============================================================================

PROJECT_NAME := polyglid-server
PROJECT_LANGUAGE := rust
PROJECT_DIR := polyglid-server
PROJECT_PATH := apps/server

.PHONY: polyglid-server-build polyglid-server-test polyglid-server-clean

polyglid-server-build:
	@echo "  Building polyglid-server..."
	@cd $(PROJECT_PATH) && $(call get_build_command,$(PROJECT_LANGUAGE))

polyglid-server-test:
	@echo "  Testing polyglid-server..."
	@cd $(PROJECT_PATH) && $(call get_test_command,$(PROJECT_LANGUAGE))

polyglid-server-clean:
	@echo "  Cleaning polyglid-server..."
	@cd $(PROJECT_PATH) && $(call get_clean_command,$(PROJECT_LANGUAGE))

polyglid-server-dev:
	@echo "  Starting polyglid-server development server..."
	@cd $(PROJECT_PATH) && $(call get_dev_command,$(PROJECT_LANGUAGE))
