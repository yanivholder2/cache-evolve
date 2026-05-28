---
name: caffeine-sim
description: Build dash.dll and run the Caffeine cache simulator. Use when the user wants to run caffeine, test with caffeine, build and run the simulator, test logs, verify JNI integration, or do a single simulation run. Triggers on "run caffeine", "test with caffeine", "caffeine run", "test the logs", "build and run", "run the simulator", "simulate".
---

Single-trace × single-size Caffeine run. Wraps `scripts/caffeine_runner.py run`.

Read `docs/caffeine-simulator.md` for the full invocation reference (JVM args, config keys per target, output files).

## Steps

1. **Resolve the cdylib dir** — must come from the current repo root (worktrees included):
   ```bash
   REPO_ROOT=$(git rev-parse --show-toplevel)
   LIB_DIR="$REPO_ROOT/target/release"
   ```

2. **Build** (background subagent): `cargo build --release`. Stop on failure.

3. **Run a single (trace, size)** via the shared runner:
   ```bash
   python "$REPO_ROOT/scripts/caffeine_runner.py" run \
       --target dash \
       --library-path "$LIB_DIR" \
       --trace lirs:multi1.trace.gz \
       --size 16384 \
       --policies classiclru
   ```
   Only one target: `dash` (cdylib `libdash.so`/`dash.dll`/`libdash.dylib`). Override `--normal-buckets`, `--stash-buckets`, `--bucket-size` to set geometry (must satisfy `segments × (normal + stash) × bucket_size == size`).

## Gotchas

- In a git worktree, `target/release/` is inside the worktree root. Always use `git rev-parse --show-toplevel`.
- JNI logs land in `rust-logs/` under the JVM cwd (typically `~/projects/caffeine/simulator/rust-logs/`).
