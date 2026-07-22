# Deprecated compatibility entry point.
# Use the repository-root Makefile or scripts/ops/polyglid-ops.mjs directly.

THIS_MAKEFILE := $(lastword $(MAKEFILE_LIST))
ROOT := $(abspath $(dir $(THIS_MAKEFILE))/../..)
ARGS ?=
BASE ?= main
HEAD ?= HEAD

COMMANDS := help init doctor dev desktop server format check validate build test clean detect graph site mvp repo-sync

.DEFAULT_GOAL := help
.PHONY: $(COMMANDS)

$(COMMANDS):
	@printf '%s\n' 'tools/automation/Makefile is deprecated; delegating to the root Makefile.' >&2
	@$(MAKE) --no-print-directory -C "$(ROOT)" "$@" ARGS="$(ARGS)" BASE="$(BASE)" HEAD="$(HEAD)"
