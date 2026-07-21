# ============================================================================
# polyglid-config - Project Makefile
# ============================================================================

PROJECT_NAME := polyglid-config
PROJECT_LANGUAGE := rust
PROJECT_DIR := polyglid-config
PROJECT_PATH := crates/config

.PHONY: polyglid-config-build polyglid-config-test polyglid-config-clean

polyglid-config-build:
	@echo "  Building polyglid-config..."
	@cd $(PROJECT_PATH) && $(call get_build_command,$(PROJECT_LANGUAGE))

polyglid-config-test:
	@echo "  Testing polyglid-config..."
	@cd $(PROJECT_PATH) && $(call get_test_command,$(PROJECT_LANGUAGE))

polyglid-config-clean:
	@echo "  Cleaning polyglid-config..."
	@cd $(PROJECT_PATH) && $(call get_clean_command,$(PROJECT_LANGUAGE))

polyglid-config-dev:
	@echo "  Starting polyglid-config development server..."
	@cd $(PROJECT_PATH) && $(call get_dev_command,$(PROJECT_LANGUAGE))
