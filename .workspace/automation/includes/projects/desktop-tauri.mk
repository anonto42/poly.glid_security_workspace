# ============================================================================
# desktop-tauri - Project Makefile
# ============================================================================

PROJECT_NAME := desktop-tauri
PROJECT_LANGUAGE := node
PROJECT_TYPE := node
PROJECT_PATH := projects/node/node/desktop-tauri

.PHONY: desktop-tauri-build desktop-tauri-test desktop-tauri-clean

desktop-tauri-build:
	@echo "  Building desktop-tauri..."
	@cd $(PROJECT_PATH) && $(call get_build_command,$(PROJECT_LANGUAGE))

desktop-tauri-test:
	@echo "  Testing desktop-tauri..."
	@cd $(PROJECT_PATH) && $(call get_test_command,$(PROJECT_LANGUAGE))

desktop-tauri-clean:
	@echo "  Cleaning desktop-tauri..."
	@cd $(PROJECT_PATH) && $(call get_clean_command,$(PROJECT_LANGUAGE))

desktop-tauri-dev:
	@echo "  Starting desktop-tauri development server..."
	@cd $(PROJECT_PATH) && $(call get_dev_command,$(PROJECT_LANGUAGE))
