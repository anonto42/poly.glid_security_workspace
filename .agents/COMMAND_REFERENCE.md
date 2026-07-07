# PolyGlid Workspace — Command Reference & Reality Mapping

## Status Key

| Icon | Meaning |
|------|---------|
| ✅ **Live** | Command exists and has real implementation |
| ⚠️ **Stub** | Target exists but is placeholder / incomplete |
| ❌ **Missing** | Target does not exist in Makefile |
| 🔧 **Needs Work** | Exists but implementation is broken or minimal |

---

## 1. `make init` — ✅ Live

**Actual implementation** (in `Makefile:44-51`):
```makefile
init:
    @$(MAKE) _install-deps
    @.workspace/automation/scripts/validate-workspace.sh
```

**Steps:**
1. Installs Node deps (`pnpm install` for react-web, `npm install` for desktop-tauri)
2. Runs `validate-workspace.sh` which checks `projects/` and `docs/` dirs exist + `Makefile` and `workspace.toml` files exist

**Gap vs hypothetical:** Hypo claims it installs `poetry`, `just`, `task`, `docker`, `kubectl`, Python/Go deps — none of those actually happen. It only installs Node deps and validates structure.

| Step | Actual | Hypo Claim |
|------|--------|------------|
| Install tools | ❌ None | poetry, just, task, docker, kubectl |
| Node deps | ✅ pnpm + npm | ✅ pnpm + npm |
| Python deps | ❌ Skipped | ❌ No Python projects |
| Rust deps | ❌ Not included | Cargo build would add |
| Go deps | ❌ Skipped | ❌ No Go projects |
| Validate | ✅ Basic dir/file check | Full project validation |

**To make it match hypo:** Add Rust tool check (`rustup`, `cargo`), Docker check, and validate all project dirs from `workspace.toml`.

---

## 2. `make status` — ✅ Live

**Actual implementation** (in `Makefile:59-65`):
```makefile
status:
    @echo "Workspace root: $(WORKSPACE_ROOT)"
    @echo "Languages enabled: $(LANGUAGES)"
    @.workspace/automation/scripts/validate-workspace.sh --quiet
```

**Reality:** Prints root path and enabled languages (`node rust`). Then runs validation script. No project health checks, no Docker/service status, no test results.

**Gap vs hypothetical:** Hypo shows 12 projects with per-service health status and Docker container counts. Actual shows only 2 fields + basic validation.

---

## 3. `make graph` — ⚠️ Stub (broken)

**Actual implementation** (in `Makefile:67-70`):
```makefile
graph:
    @.workspace/automation/scripts/generate-graph.sh
```

**Script** (`generate-graph.sh`):
- Tries to parse `[dependencies]` section from `workspace.toml` with `grep`
- **Bug:** `workspace.toml` has no `[dependencies]` section — it uses `[projects]` with inline path/language/type/dependencies fields
- Outputs a DOT graph but will produce no edges

**To fix:** Rewrite script to parse `workspace.toml` `[projects]` sections and extract `dependencies` from each project entry (e.g. `react-web = { ..., dependencies = [...] }`). But `workspace.toml` currently doesn't define inter-project dependencies for any project.

---

## 4. `make info` — ✅ Live

**Actual implementation** (in `Makefile:72-80`):
```makefile
info:
    @echo "Name: $(WORKSPACE_NAME)"
    @echo "Version: $(WORKSPACE_VERSION)"
    @echo "Root: $(WORKSPACE_ROOT)"
    @echo "Languages: $(LANGUAGES)"
```

**Reality:** Prints 5 fields from `config.mk`. No project counts, no tool versions, no AI engine status.

---

## 5. `make dev` — ✅ Live

**Actual implementation** (in `Makefile:87-100`):
```makefile
dev:
    @$(MAKE) -j$(PARALLEL_JOBS) dev-node dev-rust

dev-node:
    @(cd projects/node/react-web && npx pnpm run dev) &
    @(cd projects/node/desktop-tauri && npm run dev) &

dev-rust:
    @cargo run -p polyglid-server &
```

**Reality:** Starts 3 processes in background:
- React web dev server (Vite)
- Tauri desktop dev
- Rust backend (`polyglid-server`)

**Gap vs hypothetical:** No Python or Go services started (none exist). No health check, no unified status output.

---

## 6. `make build` — ✅ Live

**Actual implementation** (in `Makefile:107-120`):
```makefile
build:
    @$(MAKE) build-rust build-node

build-node:
    @cd projects/node/react-web && npx pnpm run build
    @cd projects/node/desktop-tauri && npm run build

build-rust:
    @cargo build --release
```

**Reality:**
- Rust: builds entire workspace (all crates) with `--release`
- Node: builds react-web + desktop-tauri

**Gap vs hypothetical:** No Python or Go builds. No per-project artifact listing.

---

## 7. `make test` — ⚠️ Partial

**Actual implementation** (in `Makefile:126-134`):
```makefile
test:
    @$(MAKE) test-rust

test-rust:
    @cargo test --all
```

**Reality:** Only Rust tests run (`cargo test --all`). No Node tests are included despite `languages.mk` defining `node-test`.

**Bug:** `test` does not call `test-node`. The `languages.mk` has `node-test` but `Makefile` test target only calls `test-rust`.

**Gap vs hypothetical:** Hypo claims 248 tests across Rust/Node/Python/Go with timing. Actual runs only Rust tests (likely far fewer).

---

## 8. `make clean` — ✅ Live

**Actual implementation** (in `Makefile:140-155`):
```makefile
clean:
    @$(MAKE) clean-node clean-rust

clean-node:
    @rm -rf projects/node/react-web/node_modules projects/node/react-web/dist
    @rm -rf projects/node/desktop-tauri/node_modules projects/node/desktop-tauri/dist

clean-rust:
    @cargo clean
```

**Reality:** Removes Node `node_modules`/`dist` dirs and runs `cargo clean`. No Python/Go cleanup (no projects). No cache cleaning.

---

## 9. `make help` — ✅ Live

**Actual implementation** (in `help.mk:5-8`):
```makefile
help:
    @grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | ...
```

**Reality:** Scans all Makefiles for targets with `##` comments and prints them formatted. Works correctly.

---

## 10. `make docker-up` / `docker-down` — ⚠️ Stub

**Actual** (in `docker.mk`):
```makefile
docker-up:
    @echo "🐳 Starting local services via docker-compose..."

docker-down:
    @echo "🐳 Stopping local services..."
```

**Reality:** Prints message only. No actual `docker-compose up/down` commands. The `infrastructure/docker/compose/` directory exists but has no `docker-compose.yml` files.

---

## 11. `make k8s-apply` / `k8s-delete` — ⚠️ Stub

**Actual** (in `k8s.mk`):
```makefile
k8s-apply:
    @echo "☸️ Applying Kubernetes manifests..."

k8s-delete:
    @echo "☸️ Deleting Kubernetes resources..."
```

**Reality:** Prints message only. No actual `kubectl` commands. The `infrastructure/docker/k8s/` directory exists but has no manifests.

---

## 12. ❌ Commands That Don't Exist

| Command | Status | Notes |
|---------|--------|-------|
| `make ai-analyze` | ❌ Missing | AI binary exists at `.workspace/ai/rust/` but no Makefile target |
| `make ai-security` | ❌ Missing | `polyglid-ai security` works directly but no make wrapper |
| `make ai-suggest` | ❌ Missing | `polyglid-ai suggest` works directly but no make wrapper |
| `make deploy` | ❌ Missing | No deploy pipeline exists |
| `make new-project` | ❌ Missing | No scaffolding script |
| `make test-node` | ❌ Missing | Target defined in `languages.mk` but never called |

---

## 13. Available AI Binary Commands (direct usage)

The AI engine crate at `.workspace/ai/rust/` provides:
```
polyglid-ai analyze       # Analyze workspace
polyglid-ai analyze -f X  # Analyze specific file
polyglid-ai suggest       # Get AI suggestions
polyglid-ai review X      # Review code file
polyglid-ai generate ...  # Generate code/tests/docs
polyglid-ai security      # Security scan
polyglid-ai optimize      # Build optimization
polyglid-ai status        # AI engine status
```

These work **directly** but have no Makefile wrappers yet.

---

## 14. Summary: Live vs Stub vs Missing

```
make help        ✅ Live     make status      ✅ Live
make init        ✅ Live     make info        ✅ Live
make dev         ✅ Live     make build       ✅ Live
make test        ⚠️ Partial  make clean       ✅ Live
make graph       ⚠️ Broken   make docker-up   ⚠️ Stub
make docker-down ⚠️ Stub     make k8s-apply   ⚠️ Stub
make k8s-delete  ⚠️ Stub     make ci-build    ⚠️ Stub
make ci-test     ⚠️ Stub     make ai-*        ❌ Missing
make deploy      ❌ Missing  make new-project ❌ Missing
```

## 15. Implementation Plan

### High Priority (fill gaps in existing targets)
1. **Fix `test`** — add `test-node` to the dependency chain
2. **Fix `graph`** — rewrite `generate-graph.sh` to parse `workspace.toml` `[projects]` properly
3. **Fix `init`** — add tool checks (rustup, cargo, node, npm/pnpm) and Rust dependency installation

### Medium Priority (turn stubs into real commands)
4. **`docker-up`/`docker-down`** — create `infrastructure/docker/compose/docker-compose.yml` and wire it up
5. **`k8s-apply`/`k8s-delete`** — create K8s manifests or reference existing ones
6. **`ai-analyze`/`ai-suggest`/`ai-security`** — add Makefile wrappers calling `polyglid-ai` binary

### Low Priority (new functionality)
7. **`deploy`** — deployment pipeline (depends on Docker + K8s being real first)
8. **`new-project`** — scaffolding script
9. **`test-node`** — wire up as separate target
