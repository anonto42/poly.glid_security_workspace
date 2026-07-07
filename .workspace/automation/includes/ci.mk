# ============================================================================
# CI/CD Commands
# ============================================================================

.PHONY: ci-build ci-test

ci-build:
	@echo "🔨 CI Build started..."
	$(MAKE) build

ci-test:
	@echo "🧪 CI Test started..."
	$(MAKE) test
