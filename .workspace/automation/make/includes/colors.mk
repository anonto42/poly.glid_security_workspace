# ============================================================================
# Color Definitions
# ============================================================================

# Basic colors
BLACK   := \033[0;30m
RED     := \033[0;31m
GREEN   := \033[0;32m
YELLOW  := \033[0;33m
BLUE    := \033[0;34m
PURPLE  := \033[0;35m
CYAN    := \033[0;36m
WHITE   := \033[0;37m

# Bold colors
BOLD_BLACK   := \033[1;30m
BOLD_RED     := \033[1;31m
BOLD_GREEN   := \033[1;32m
BOLD_YELLOW  := \033[1;33m
BOLD_BLUE    := \033[1;34m
BOLD_PURPLE  := \033[1;35m
BOLD_CYAN    := \033[1;36m
BOLD_WHITE   := \033[1;37m

# Background colors
BG_BLACK   := \033[40m
BG_RED     := \033[41m
BG_GREEN   := \033[42m
BG_YELLOW  := \033[43m
BG_BLUE    := \033[44m
BG_PURPLE  := \033[45m
BG_CYAN    := \033[46m
BG_WHITE   := \033[47m

# Reset
RESET := \033[0m

# Print functions
define print_header
	@printf "\n$(BOLD_CYAN)════════════════════════════════════════════════════════════════$(RESET)\n"
	@printf "$(BOLD_CYAN)  $1$(RESET)\n"
	@printf "$(BOLD_CYAN)════════════════════════════════════════════════════════════════$(RESET)\n\n"
endef

define print_step
	@printf "$(BOLD_GREEN)▶$(RESET) $(BOLD_WHITE)$1$(RESET)\n"
endef

define print_substep
	@printf "  $(BOLD_BLUE)▸$(RESET) $1\n"
endef

define print_success
	@printf "\n$(BOLD_GREEN)✅$(RESET) $(GREEN)$1$(RESET)\n\n"
endef

define print_error
	@printf "\n$(BOLD_RED)❌$(RESET) $(RED)$1$(RESET)\n\n"
endef

define print_warning
	@printf "\n$(BOLD_YELLOW)⚠️$(RESET) $(YELLOW)$1$(RESET)\n\n"
endef

define print_info
	@printf "$(BOLD_CYAN)ℹ️$(RESET) $(CYAN)$1$(RESET)\n"
endef
