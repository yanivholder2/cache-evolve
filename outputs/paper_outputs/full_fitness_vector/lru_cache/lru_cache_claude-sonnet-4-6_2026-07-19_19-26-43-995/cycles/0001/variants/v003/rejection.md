
## 2026-07-19T16:37:04Z — variant=cycle/0001/v003 hypothesis=H03 attempt=1

**Gate**: build gate
**Error**:
```
    Updating crates.io index
     Locking 86 packages to latest compatible versions
      Adding jni v0.20.0 (available: v0.22.4)
      Adding rand v0.8.7 (available: v0.10.2)
      Adding simplelog v0.11.2 (available: v0.12.2)
      Adding termcolor v1.1.3 (available: v1.4.1)
   Compiling cfg-if v1.0.4
   Compiling windows-link v0.2.1
   Compiling proc-macro2 v1.0.107
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.47
   Compiling zerocopy v0.8.54
   Compiling getrandom v0.3.4
   Compiling autocfg v1.5.1
   Compiling windows-sys v0.61.2
   Compiling getrandom v0.4.3
   Compiling jni-sys v0.3.1
   Compiling fastrand v2.5.0
   Compiling thiserror v1.0.69
   Compiling num-traits v0.2.19
   Compiling rand_core v0.9.5
   Compiling once_cell v1.21.4
   Compiling bytes v1.12.1
   Compiling quick-error v1.2.3
   Compiling bit-vec v0.8.0
   Compiling log v0.4.33
   Compiling winapi-util v0.1.11
   Compiling same-file v1.0.6
   Compiling walkdir v2.5.0
   Compiling tempfile v3.27.0
   Compiling wait-timeout v0.2.1
   Compiling syn v2.0.119
   Compiling fnv v1.0.7
   Compiling memchr v2.8.3
   Compiling rusty-fork v0.3.1
   Compiling jni v0.20.0
   Compiling termcolor v1.1.3
   Compiling chrono v0.4.45
   Compiling bit-set v0.8.0
   Compiling combine v4.6.7
   Compiling rand_xorshift v0.4.0
   Compiling rand v0.9.5
   Compiling unarray v0.1.4
   Compiling regex-syntax v0.8.11
   Compiling cesu8 v1.1.0
   Compiling bitflags v2.13.1
   Compiling getrandom v0.2.17
   Compiling simplelog v0.11.2
   Compiling framework v0.1.0 (C:\Users\yaniv\Projects\cache-evolve\framework-rs)
   Compiling rand_core v0.6.4
   Compiling ppv-lite86 v0.2.21
   Compiling jni-sys-macros v0.4.1
   Compiling thiserror-impl v1.0.69
   Compiling rand_chacha v0.9.0
   Compiling rand_chacha v0.3.1
   Compiling jni-sys v0.4.1
   Compiling proptest v1.11.0
   Compiling rand v0.8.7
   Compiling lru_cache v0.1.0 (C:\Users\yaniv\AppData\Local\Temp\variant-workspace-phpa1msc)
error[E0432]: unresolved import `lru_cache::lru_cache::Node`
 --> src\bin\evolve_metadata_size.rs:9:38
  |
9 | use lru_cache::lru_cache::{LruCache, Node};
  |                                      ^^^^ no `Node` in `lru_cache`

For more information about this error, try `rustc --explain E0432`.
error: could not compile `lru_cache` (bin "lru_cache-evolve-metadata-size") due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `lru_cache` (bin "lru_cache-evolve-metadata-size" test) due to 1 previous error

```
---
