# ============================================================================
# Help Target
# ============================================================================

.PHONY: help
help: ## Show this help message
	@$(call print_header,🛠️ Available Workspace Commands)
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
