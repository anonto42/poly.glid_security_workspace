# ============================================================================
# Utility Functions
# ============================================================================

# Check if a command exists
define command_exists
	@command -v $(1) >/dev/null 2>&1
endef

# Get project type from path
define get_project_type
	$(shell echo $(1) | cut -d'/' -f3)
endef

# Get language from project
define get_project_language
	$(shell echo $(1) | cut -d'/' -f2)
endef

# Get build command for project
define get_build_command
	$(shell case "$(1)" in \
		node*) echo "pnpm build" ;; \
		rust*) echo "cargo build --release" ;; \
		*) echo "echo 'No build command'" ;; \
	esac)
endef

# Get test command for project
define get_test_command
	$(shell case "$(1)" in \
		node*) echo "pnpm test" ;; \
		rust*) echo "cargo test" ;; \
		*) echo "echo 'No test command'" ;; \
	esac)
endef

# Get clean command for project
define get_clean_command
	$(shell case "$(1)" in \
		node*) echo "rm -rf node_modules dist" ;; \
		rust*) echo "cargo clean" ;; \
		*) echo "echo 'No clean command'" ;; \
	esac)
endef

# Get dev command for project
define get_dev_command
	$(shell case "$(1)" in \
		node*) echo "npm run dev" ;; \
		rust*) echo "cargo run" ;; \
		*) echo "echo 'No dev command'" ;; \
	esac)
endef

# Get current timestamp
define timestamp
	$(shell date +%Y%m%d_%H%M%S)
endef

# Create directory if it doesn't exist
define ensure_dir
	@mkdir -p $(1)
endef

# Log message
define log
	@echo "[$(shell date +%H:%M:%S)] $(1)" >> $(LOGS_DIR)/workspace.log
endef
