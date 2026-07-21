# ============================================================================
# OS Detection Layer — runs before every make target
# ============================================================================

# Detect OS
UNAME_S := $(shell uname -s)

# Normalized OS name
ifeq ($(UNAME_S),Linux)
  OS := linux
else ifeq ($(UNAME_S),Darwin)
  OS := macos
else ifneq (,$(findstring MINGW,$(UNAME_S)))
  OS := windows
else ifneq (,$(findstring MSYS,$(UNAME_S)))
  OS := windows
else
  OS := unknown
endif

# Boolean flags for conditional branching
IS_LINUX   := $(if $(filter linux,$(OS)),true,false)
IS_MACOS   := $(if $(filter macos,$(OS)),true,false)
IS_WINDOWS := $(if $(filter windows,$(OS)),true,false)

# Architecture
UNAME_M := $(shell uname -m)

# CPU count (OS-aware)
ifeq ($(OS),linux)
  PARALLEL_JOBS := $(shell nproc 2>/dev/null || echo 4)
else ifeq ($(OS),macos)
  PARALLEL_JOBS := $(shell sysctl -n hw.logicalcpu 2>/dev/null || echo 4)
else ifeq ($(OS),windows)
  PARALLEL_JOBS := $(shell echo %NUMBER_OF_PROCESSORS% 2>/dev/null || echo 4)
else
  PARALLEL_JOBS := 4
endif

# Path separator
ifeq ($(OS),windows)
  PATH_SEP := \\
  PATH_SEP_NATIVE := ;
else
  PATH_SEP := /
  PATH_SEP_NATIVE := :
endif

# Null device
ifeq ($(OS),windows)
  NULL_DEV := NUL
else
  NULL_DEV := /dev/null
endif

# Command to check if a binary exists
ifeq ($(OS),windows)
  CHECK_CMD := where
else
  CHECK_CMD := command -v
endif

# ============================================================================
# Shell Selection — OS-appropriate shell for all recipes
# ============================================================================
#
# Linux / macOS : native /bin/bash
# Windows        : try Git Bash; fall back to cmd.exe with a warning.
#
# All recipes use bash syntax. On Windows, install Git Bash or WSL.

ifeq ($(OS),windows)
  SHELL := $(COMSPEC)
  .SHELLFLAGS := /C
  $(warning Windows detected: recipes assume bash syntax. Install Git Bash or use WSL for full compatibility.)
else
  SHELL := /bin/bash
  .SHELLFLAGS := -eu -o pipefail -c
endif
