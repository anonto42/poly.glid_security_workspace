# ============================================================================
# polyglid-events - Project Makefile
# ============================================================================

PROJECT_NAME := polyglid-events
PROJECT_LANGUAGE := rust
PROJECT_TYPE := rust/crates
PROJECT_PATH := projects/rust/rust/crates/polyglid-events

.PHONY: polyglid-events-build polyglid-events-test polyglid-events-clean

polyglid-events-build:
	@echo "  Building polyglid-events..."
	@cd $(PROJECT_PATH) && $(call get_build_command,$(PROJECT_LANGUAGE))

polyglid-events-test:
	@echo "  Testing polyglid-events..."
	@cd $(PROJECT_PATH) && $(call get_test_command,$(PROJECT_LANGUAGE))

polyglid-events-clean:
	@echo "  Cleaning polyglid-events..."
	@cd $(PROJECT_PATH) && $(call get_clean_command,$(PROJECT_LANGUAGE))

polyglid-events-dev:
	@echo "  Starting polyglid-events development server..."
	@cd $(PROJECT_PATH) && $(call get_dev_command,$(PROJECT_LANGUAGE))
