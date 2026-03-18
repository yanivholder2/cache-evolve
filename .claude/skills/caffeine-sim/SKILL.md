---
description: Build dash.dll and run the Caffeine cache simulator
model: opus
allowed-tools: Bash(*gradlew*), Agent
---

Build the Rust JNI library and run the Caffeine cache simulator.

1. Build in a background subagent: launch an Agent with `run_in_background: true` to run `cargo build --release`. Use `allowed-tools: Bash(cargo build *)` for the subagent.
2. Wait for the build to complete. If it fails, report the error and stop.
3. Run the simulator (10-minute timeout):
   ```bash
   "C:/Users/yanivholder/Downloads/ThesisNew/caffeine/gradlew" -p "C:/Users/yanivholder/Downloads/ThesisNew/caffeine" simulator:run -PjvmArgs="-XX:+UseParallelGC,-Xmx8g,-Djava.library.path=C:/Users/yanivholder/Downloads/ThesisNew/dash-cache-rs/target/release" -q
   ```
