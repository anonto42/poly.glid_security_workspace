# ============================================================================
# Workspace Configuration
# ============================================================================

# Workspace root
WORKSPACE_ROOT := $(shell pwd)
WORKSPACE_NAME := polyglid-workspace
WORKSPACE_VERSION := 1.0.0

# Languages
LANGUAGES := node rust

# Cache directories
CACHE_DIR := .workspace/state/cache
LOGS_DIR := .workspace/state/logs
TEMP_DIR := .workspace/state/temp

# Build flags
BUILD_FLAGS := --release
TEST_FLAGS := --verbose

# Docker
DOCKER_REGISTRY := docker.io/polyglid
DOCKER_TAG := latest

# Kubernetes
K8S_NAMESPACE := polyglid

# Service discovery
CONSUL_ADDR := consul:8500

# Environment
ENV := development
