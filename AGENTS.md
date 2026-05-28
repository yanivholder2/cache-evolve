# Docs

@docs/docs-map.md

# Source Control & Workflow
After completing a logical unit of work (feature, fix, refactor, etc.), propose a commit. Avoid staging/committing changes unless explicitly requested.

# Evolve Framework

Research framework at the repo root that drives the cache-policy evolution loop (cycles, cohorts, gates, fitness, ledger). See `README.md` for details.

- Always use the venv python: `.venv/bin/python` (never system `python`/`python3`).
- Run `uv sync` to create the venv and install dependencies if missing.
- Testing: keep the feedback loop tight. Run the fast suite first (no opt-in flags), then run each slower marker one at a time — never combine markers in a single run. See `README.md` for the current marker flags.

# Caffeine Simulator Integration
Dash cache integrates with [Caffeine](https://github.com/ben-manes/caffeine)'s cache simulator via JNI.

**Repo**: `~/projects/caffeine` (sibling of the main repo; the only caffeine repo to use)

**Invariants**:
- Pointer-based JNI: Java side stores the pointer returned from cache init and passes it back on every call.
- Eviction-policy ordinals must match Rust `EvictionPolicyKind::from_usize` indices.
- Java settings inner classes read `dash.*` config from `application.conf`.

**Build + Run workflow**:
1. `cargo build --release` (produces `target/release/dash.dll`)
2. `cargo test`
3. (Optional, only if requested) From caffeine dir: `./gradlew simulator:run -PjvmArgs="-XX:+UseParallelGC,-Xmx8g,-Djava.library.path=<path-to-cache-evolve>/target/release" -q`
