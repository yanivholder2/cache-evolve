# Docs

@docs/docs-map.md

# Research Context
M.Sc. thesis on mitigating the **hot set problem** in N-way associative caches using Dash-inspired bucket movement. Items can migrate between adjacent buckets to reduce contention in overloaded sets. See `docs/research-proposal.tex` for full proposal, motivation, and initial results.

# Source Control & Workflow
After completing a logical unit of work (feature, fix, refactor, etc.), propose a commit. Avoid staging/committing changes unless explicitly requested.

# Evolve Framework

Research framework under `evolve/` that drives the cache-policy evolution loop (cycles, cohorts, gates, fitness, ledger). See `evolve/README.md` for details.

- Always use the venv python: `evolve/.venv/bin/python` (never system `python`/`python3`).
- Run `evolve/scripts/setup-evolve.sh` to create the venv and install `requirements.txt` if missing.
- Testing: keep the feedback loop tight. Run the fast suite first (no opt-in flags), then run each slower marker one at a time — never combine markers in a single run. See `evolve/README.md` for the current marker flags.

# Caffeine Simulator Integration
Dash cache integrates with [Caffeine](https://github.com/ben-manes/caffeine)'s cache simulator via JNI.

**Repo**: `~/projects/caffeine` (sibling of the main repo; the only caffeine repo to use)

**Invariants**:
- Pointer-based JNI: Java side stores the pointer returned from cache init and passes it back on every call.
- Eviction-policy enum ordinals must match Rust `EvictionPolicy::from_usize` indices.
- Java settings inner classes read `dash.*` config from `application.conf`.

**Build + Run workflow**:
1. `cargo build --release` (produces `target/release/dash.dll`)
2. `cargo test`
3. (Optional, only if requested) From caffeine dir: `./gradlew simulator:run -PjvmArgs="-XX:+UseParallelGC,-Xmx8g,-Djava.library.path=<path-to-dash-cache-rs>/target/release" -q`
