
## 2026-06-05T18:37:34Z — variant=cycle/0003/v001 hypothesis=H01 attempt=1

**Gate**: build gate
**Error**:
```
    Updating crates.io index
     Locking 114 packages to latest compatible versions
      Adding jni v0.20.0 (available: v0.22.4)
      Adding rand v0.8.6 (available: v0.10.1)
      Adding simplelog v0.11.2 (available: v0.12.2)
      Adding termcolor v1.1.3 (available: v1.4.1)
   Compiling libc v0.2.186
   Compiling cfg-if v1.0.4
   Compiling proc-macro2 v1.0.106
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.45
   Compiling zerocopy v0.8.50
   Compiling getrandom v0.3.4
   Compiling autocfg v1.5.1
   Compiling syn v2.0.117
   Compiling num-traits v0.2.19
   Compiling getrandom v0.4.2
   Compiling rustix v1.1.4
   Compiling rand_core v0.9.5
   Compiling same-file v1.0.6
   Compiling linux-raw-sys v0.12.1
   Compiling bitflags v2.12.1
   Compiling walkdir v2.5.0
   Compiling getrandom v0.2.17
   Compiling jni-sys v0.3.1
   Compiling once_cell v1.21.4
   Compiling fastrand v2.4.1
   Compiling thiserror v1.0.69
   Compiling tempfile v3.27.0
   Compiling rand_core v0.6.4
   Compiling ppv-lite86 v0.2.21
   Compiling jni v0.20.0
   Compiling wait-timeout v0.2.1
   Compiling iana-time-zone v0.1.65
   Compiling fnv v1.0.7
   Compiling bytes v1.11.1
   Compiling memchr v2.8.1
   Compiling quick-error v1.2.3
   Compiling bit-vec v0.8.0
   Compiling jni-sys-macros v0.4.1
   Compiling thiserror-impl v1.0.69
   Compiling jni-sys v0.4.1
   Compiling log v0.4.32
   Compiling combine v4.6.7
   Compiling bit-set v0.8.0
   Compiling rusty-fork v0.3.1
   Compiling rand_chacha v0.9.0
   Compiling rand_chacha v0.3.1
   Compiling chrono v0.4.45
   Compiling rand_xorshift v0.4.0
   Compiling rand v0.9.4
   Compiling termcolor v1.1.3
   Compiling regex-syntax v0.8.10
   Compiling unarray v0.1.4
   Compiling cesu8 v1.1.0
   Compiling simplelog v0.11.2
   Compiling rand v0.8.6
   Compiling framework v0.1.0 (/home/user/cache-evolve/framework-rs)
   Compiling proptest v1.11.0
   Compiling dash v0.1.0 (/tmp/variant-workspace-m6zugbhw)
error: right curly brace `}` before `else` in a `let...else` statement not allowed
   --> src/dash/segments/dash_segment.rs:176:3
    |
176 |         } else {
    |         ^
    |
help: wrap the expression in parentheses
    |
172 ~         let Some(item_new_idx) = (if occupancy_pct >= 0.75 {
173 |             core.buckets[probing_idx].get_position(key)
174 |         } else {
175 |             core.buckets[target_idx].get_position(key)
176 ~         }) else {
    |

error: could not compile `dash` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `dash` (lib test) due to 1 previous error

```
---
