# ============================================================================
# recon_probe - Project Makefile
# ============================================================================

PROJECT_NAME := recon_probe
PROJECT_LANGUAGE := rust
PROJECT_TYPE := rust/plugins
PROJECT_PATH := projects/rust/rust/plugins/recon_probe

.PHONY: recon_probe-build recon_probe-test recon_probe-clean

recon_probe-build:
	@echo "  Building recon_probe..."
	@cd $(PROJECT_PATH) && $(call get_build_command,$(PROJECT_LANGUAGE))

recon_probe-test:
	@echo "  Testing recon_probe..."
	@cd $(PROJECT_PATH) && $(call get_test_command,$(PROJECT_LANGUAGE))

recon_probe-clean:
	@echo "  Cleaning recon_probe..."
	@cd $(PROJECT_PATH) && $(call get_clean_command,$(PROJECT_LANGUAGE))

recon_probe-dev:
	@echo "  Starting recon_probe development server..."
	@cd $(PROJECT_PATH) && $(call get_dev_command,$(PROJECT_LANGUAGE))
