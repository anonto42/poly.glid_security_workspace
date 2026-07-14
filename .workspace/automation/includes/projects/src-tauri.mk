# ============================================================================
# src-tauri - Project Makefile
# ============================================================================

PROJECT_NAME := src-tauri
PROJECT_LANGUAGE := rust
PROJECT_DIR := polyglid-desktop-legacy/src-tauri
PROJECT_PATH := projects/polyglid-desktop-legacy/src-tauri

.PHONY: src-tauri-build src-tauri-test src-tauri-clean

src-tauri-build:
	@echo "  Building src-tauri..."
	@cd $(PROJECT_PATH) && $(call get_build_command,$(PROJECT_LANGUAGE))

src-tauri-test:
	@echo "  Testing src-tauri..."
	@cd $(PROJECT_PATH) && $(call get_test_command,$(PROJECT_LANGUAGE))

src-tauri-clean:
	@echo "  Cleaning src-tauri..."
	@cd $(PROJECT_PATH) && $(call get_clean_command,$(PROJECT_LANGUAGE))

src-tauri-dev:
	@echo "  Starting src-tauri development server..."
	@cd $(PROJECT_PATH) && $(call get_dev_command,$(PROJECT_LANGUAGE))
