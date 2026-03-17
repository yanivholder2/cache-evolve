---
description: Build dynamic library (.dll) for JNI integration
disable-model-invocation: true
model: opus
---

Build the project as a dynamic library for JNI integration.

Steps:
1. Uncomment `crate_type = ["cdylib"]` in Cargo.toml (remove the `#` prefix)
2. Run `cargo build --release`
3. Report the path to the built .dll file in `target/release/`
4. Re-comment the `crate_type` line (add `#` prefix back) so binary mode works again
