---
name: caffeine-sim
description: Build dash.dll and run the Caffeine cache simulator. Use when the user wants to run caffeine, test with caffeine, build and run the simulator, test logs, verify JNI integration, or do a single simulation run. Triggers on "run caffeine", "test with caffeine", "caffeine run", "test the logs", "build and run", "run the simulator", "simulate".
---

Build the Rust JNI library and run the Caffeine cache simulator.

## Steps

1. **Resolve the library path** — the DLL must come from the current repo root (which may be a worktree):
   ```bash
   REPO_ROOT=$(git rev-parse --show-toplevel)
   ```
   Use `$REPO_ROOT/target/release` as the `java.library.path`.

2. **Build**: Run `cargo build --release`. Run this as a background subagent.

3. Wait for the build to complete. If it fails, report the error and stop.

4. **Run the simulator** (10-minute timeout):
   ```bash
   REPO_ROOT=$(git rev-parse --show-toplevel)
   CAFFEINE_DIR=~/projects/caffeine
   "$CAFFEINE_DIR/gradlew" -p "$CAFFEINE_DIR" simulator:run -PjvmArgs="-XX:+UseParallelGC,-Xmx8g,-Djava.library.path=$REPO_ROOT/target/release" -q
   ```

## Gotchas

- In a git worktree, `target/release/` is inside the worktree root, not the main repo. Always use `git rev-parse --show-toplevel` to get the correct path.
- Log files are written to `rust-logs/` relative to the JVM working directory (typically `caffeine/simulator/rust-logs/`).
