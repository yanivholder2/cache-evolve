# Docs

@docs/docs-map.md

# Evolve Framework

Research framework at the repo root that drives the cache-policy evolution loop (cycles, cohorts, gates, fitness, ledger). See `README.md` for details.

- Always use the venv python: `.venv/bin/python` (never system `python`/`python3`).
- Run `uv sync` to create the venv and install dependencies if missing.
- Testing: keep the feedback loop tight. Run the fast suite first (no opt-in flags), then run each slower marker one at a time — never combine markers in a single run. See `README.md` for the current marker flags.

# Caffeine Simulator Integration
Each Subject cache integrates with [Caffeine](https://github.com/ben-manes/caffeine)'s cache simulator via JNI. Per-subject wiring lives in the subject's `manifest.toml` `[caffeine]` table; the runner is subject-agnostic.

**Repo**: `~/projects/caffeine` (sibling of the main repo; the only caffeine repo to use)

**Invariants**:
- Pointer-based JNI: Java side stores the pointer returned from cache init and passes it back on every call.
- For subjects with an eviction-policy selector, the Java ordinals must match the Rust `EvictionPolicyKind::from_usize` indices.
- Each subject's Java settings inner classes read config under its own `config-prefix` (e.g. `caffeine.simulator.dash.*`, `caffeine.simulator.associativeCache.*`, `caffeine.simulator.lruRust.*`) from `application.conf`.

**Build + Run workflow**:
1. `cargo build --release` (each subject crate produces its own cdylib, e.g. `target/release/libdash.so`, `libassociative_cache.so`, `liblru_cache.so`)
2. `cargo test`
3. (Optional, only if requested) From caffeine dir: `./gradlew simulator:run -PjvmArgs="-XX:+UseParallelGC,-Xmx8g,-Djava.library.path=<path-to-cache-evolve>/target/release" -q`
