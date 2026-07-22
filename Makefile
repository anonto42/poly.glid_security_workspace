# PolyGlid local command compatibility layer.
#
# Repository behavior belongs in scripts/ops/polyglid-ops.mjs. Keep this file
# intentionally small so Make, npm, and GitHub Actions execute the same tasks.

NODE ?= node
OPS := $(NODE) scripts/ops/polyglid-ops.mjs

ARGS ?=
BASE ?= main
HEAD ?= HEAD

.DEFAULT_GOAL := help

.PHONY: help init doctor dev dev-rust desktop server format check validate build test clean detect graph site mvp repo-sync

help:
	@$(OPS) help $(ARGS)
	@printf '\nMake aliases:\n  init -> doctor (checks prerequisites; installs nothing)\n  dev  -> desktop\n  site -> site-build\n  mvp  -> mvp-smoke\n\nForward extra arguments with ARGS="...".\nUse BASE and HEAD with detect.\n'

# Compatibility setup is deliberately read-only. Tool installation remains an
# explicit developer decision rather than a side effect of `make init`.
init:
	@$(OPS) doctor $(ARGS)

doctor:
	@$(OPS) doctor $(ARGS)

dev:
	@$(OPS) desktop $(ARGS)

# Compatibility alias retained for callers that used the old server target.
dev-rust: server

desktop:
	@$(OPS) desktop $(ARGS)

server:
	@$(OPS) server $(ARGS)

format:
	@$(OPS) format $(ARGS)

check:
	@$(OPS) check $(ARGS)

validate:
	@$(OPS) validate $(ARGS)

build:
	@$(OPS) build $(ARGS)

test:
	@$(OPS) test $(ARGS)

clean:
	@$(OPS) clean $(ARGS)

detect:
	@$(OPS) detect "$(BASE)" "$(HEAD)" $(ARGS)

graph:
	@$(OPS) graph $(ARGS)

site:
	@$(OPS) site-build $(ARGS)

mvp:
	@$(OPS) mvp-smoke $(ARGS)

repo-sync:
	@$(OPS) repo-sync $(ARGS)
