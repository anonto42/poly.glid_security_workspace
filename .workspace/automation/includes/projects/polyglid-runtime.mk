# ============================================================================
# polyglid-runtime - Project Makefile
# ============================================================================

PROJECT_NAME := polyglid-runtime
PROJECT_LANGUAGE := rust
PROJECT_DIR := polyglid-runtime
PROJECT_PATH := projects/polyglid-runtime

.PHONY: polyglid-runtime-build polyglid-runtime-test polyglid-runtime-clean

polyglid-runtime-build:
	@echo "  Building polyglid-runtime..."
	@cd $(PROJECT_PATH) && $(call get_build_command,$(PROJECT_LANGUAGE))

polyglid-runtime-test:
	@echo "  Testing polyglid-runtime..."
	@cd $(PROJECT_PATH) && $(call get_test_command,$(PROJECT_LANGUAGE))

polyglid-runtime-clean:
	@echo "  Cleaning polyglid-runtime..."
	@cd $(PROJECT_PATH) && $(call get_clean_command,$(PROJECT_LANGUAGE))

polyglid-runtime-dev:
	@echo "  Starting polyglid-runtime development server..."
	@cd $(PROJECT_PATH) && $(call get_dev_command,$(PROJECT_LANGUAGE))
