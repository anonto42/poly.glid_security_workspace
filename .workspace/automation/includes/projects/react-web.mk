# ============================================================================
# react-web - Project Makefile
# ============================================================================

PROJECT_NAME := react-web
PROJECT_LANGUAGE := node
PROJECT_TYPE := node
PROJECT_PATH := projects/node/node/react-web

.PHONY: react-web-build react-web-test react-web-clean

react-web-build:
	@echo "  Building react-web..."
	@cd $(PROJECT_PATH) && $(call get_build_command,$(PROJECT_LANGUAGE))

react-web-test:
	@echo "  Testing react-web..."
	@cd $(PROJECT_PATH) && $(call get_test_command,$(PROJECT_LANGUAGE))

react-web-clean:
	@echo "  Cleaning react-web..."
	@cd $(PROJECT_PATH) && $(call get_clean_command,$(PROJECT_LANGUAGE))

react-web-dev:
	@echo "  Starting react-web development server..."
	@cd $(PROJECT_PATH) && $(call get_dev_command,$(PROJECT_LANGUAGE))
