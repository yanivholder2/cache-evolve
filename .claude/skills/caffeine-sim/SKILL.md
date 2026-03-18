---
description: Build dash.dll and run the Caffeine cache simulator
model: opus
context: fork
allowed-tools: Bash(cargo build *), Bash(*gradlew*)
---

Build the Rust JNI library and run the Caffeine cache simulator.

1. Run `cargo build --release`
2. Run the simulator (10-minute timeout):
   ```bash
   "C:/Users/yanivholder/Downloads/ThesisNew/caffeine/gradlew" -p "C:/Users/yanivholder/Downloads/ThesisNew/caffeine" simulator:run -PjvmArgs="-XX:+UseParallelGC,-Xmx8g,-Djava.library.path=C:/Users/yanivholder/Downloads/ThesisNew/dash-cache-rs/target/release" -q
   ```

IMPORTANT: Your response MUST always include the full simulator results table exactly as printed in the output. Never omit or summarize it — the user needs to see the raw table every time.
