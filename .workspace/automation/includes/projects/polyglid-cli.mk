# ============================================================================
# polyglid-cli - Project Makefile
# ============================================================================

PROJECT_NAME := polyglid-cli
PROJECT_LANGUAGE := rust
PROJECT_DIR := polyglid-cli
PROJECT_PATH := slices/apps/cli

.PHONY: polyglid-cli-build polyglid-cli-test polyglid-cli-clean

polyglid-cli-build:
	@echo "  Building polyglid-cli..."
	@cd $(PROJECT_PATH) && $(call get_build_command,$(PROJECT_LANGUAGE))

polyglid-cli-test:
	@echo "  Testing polyglid-cli..."
	@cd $(PROJECT_PATH) && $(call get_test_command,$(PROJECT_LANGUAGE))

polyglid-cli-clean:
	@echo "  Cleaning polyglid-cli..."
	@cd $(PROJECT_PATH) && $(call get_clean_command,$(PROJECT_LANGUAGE))

polyglid-cli-dev:
	@echo "  Starting polyglid-cli development server..."
	@cd $(PROJECT_PATH) && $(call get_dev_command,$(PROJECT_LANGUAGE))
