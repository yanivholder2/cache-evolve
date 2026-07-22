
## 2026-07-22T02:12:41Z — variant=cycle/0006/v001 hypothesis=H01 attempt=1

**Gate**: build gate
**Error**:
```
    Updating crates.io index
     Locking 86 packages to latest compatible versions
      Adding jni v0.20.0 (available: v0.22.4)
      Adding rand v0.8.7 (available: v0.10.2)
      Adding simplelog v0.11.2 (available: v0.12.2)
      Adding termcolor v1.1.3 (available: v1.4.1)
   Compiling windows-link v0.2.1
   Compiling cfg-if v1.0.4
   Compiling proc-macro2 v1.0.107
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.47
   Compiling zerocopy v0.8.55
   Compiling getrandom v0.3.4
   Compiling autocfg v1.5.1
   Compiling windows-sys v0.61.2
   Compiling getrandom v0.4.3
   Compiling getrandom v0.2.17
   Compiling jni-sys v0.3.1
   Compiling num-traits v0.2.19
   Compiling thiserror v1.0.69
   Compiling rand_core v0.9.5
   Compiling winapi-util v0.1.11
   Compiling once_cell v1.21.4
   Compiling fastrand v2.5.0
   Compiling same-file v1.0.6
   Compiling tempfile v3.27.0
   Compiling walkdir v2.5.0
   Compiling syn v2.0.119
   Compiling rand_core v0.6.4
   Compiling quick-error v1.2.3
   Compiling bit-vec v0.8.0
   Compiling jni v0.20.0
   Compiling fnv v1.0.7
   Compiling memchr v2.8.3
   Compiling bytes v1.12.1
   Compiling wait-timeout v0.2.1
   Compiling log v0.4.33
   Compiling bit-set v0.8.0
   Compiling rusty-fork v0.3.1
   Compiling chrono v0.4.45
   Compiling termcolor v1.1.3
   Compiling combine v4.6.7
   Compiling rand v0.9.5
   Compiling rand_xorshift v0.4.0
   Compiling regex-syntax v0.8.11
   Compiling unarray v0.1.4
   Compiling ppv-lite86 v0.2.21
   Compiling bitflags v2.13.1
   Compiling cesu8 v1.1.0
   Compiling simplelog v0.11.2
   Compiling rand_chacha v0.9.0
   Compiling rand_chacha v0.3.1
   Compiling framework v0.1.0 (C:\Users\yaniv\Projects\cache-evolve\framework-rs)
   Compiling rand v0.8.7
   Compiling proptest v1.11.0
   Compiling jni-sys-macros v0.4.1
   Compiling thiserror-impl v1.0.69
   Compiling jni-sys v0.4.1
   Compiling associative_cache v0.1.0 (C:\Users\yaniv\AppData\Local\Temp\variant-workspace-4n_fyplv)
error[E0599]: no variant, associated function, or constant named `TimestampLRU` found for enum `shared::eviction_policy::EvictionPolicyKind` in the current scope
   --> src\shared\traits\bucket.rs:261:59
    |
261 |         let mut bucket = TestBucket::new(2, EvictionPolicyKind::TimestampLRU);
    |                                                                 ^^^^^^^^^^^^ variant, associated function, or constant not found in `shared::eviction_policy::EvictionPolicyKind`
    |
   ::: src\shared\eviction_policy.rs:121:1
    |
121 | pub enum EvictionPolicyKind {
    | --------------------------- variant, associated function, or constant `TimestampLRU` not found for this enum

error[E0599]: no variant, associated function, or constant named `Lfu` found for enum `shared::eviction_policy::EvictionPolicyKind` in the current scope
   --> src\shared\traits\bucket.rs:295:59
    |
295 |         let mut bucket = TestBucket::new(4, EvictionPolicyKind::Lfu);
    |                                                                 ^^^ variant, associated function, or constant not found in `shared::eviction_policy::EvictionPolicyKind`
    |
   ::: src\shared\eviction_policy.rs:121:1
    |
121 | pub enum EvictionPolicyKind {
    | --------------------------- variant, associated function, or constant `Lfu` not found for this enum

error[E0609]: no field `lfu_counter` on type `&item::Item<i64, i64>`
   --> src\shared\traits\bucket.rs:297:38
    |
297 |         assert_eq!(bucket.get(&1).unwrap().lfu_counter, 1);
    |                                            ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `key`, `value`

error[E0609]: no field `lfu_counter` on type `&item::Item<i64, i64>`
   --> src\shared\traits\bucket.rs:298:38
    |
298 |         assert_eq!(bucket.get(&1).unwrap().lfu_counter, 2);
    |                                            ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `key`, `value`

error[E0609]: no field `lfu_counter` on type `&item::Item<i64, i64>`
   --> src\shared\traits\bucket.rs:299:38
    |
299 |         assert_eq!(bucket.get(&1).unwrap().lfu_counter, 3);
    |                                            ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `key`, `value`

error[E0599]: no variant, associated function, or constant named `Lfu` found for enum `shared::eviction_policy::EvictionPolicyKind` in the current scope
   --> src\shared\traits\bucket.rs:304:59
    |
304 |         let mut bucket = TestBucket::new(2, EvictionPolicyKind::Lfu);
    |                                                                 ^^^ variant, associated function, or constant not found in `shared::eviction_policy::EvictionPolicyKind`
    |
   ::: src\shared\eviction_policy.rs:121:1
    |
121 | pub enum EvictionPolicyKind {
    | --------------------------- variant, associated function, or constant `Lfu` not found for this enum

error[E0599]: no variant, associated function, or constant named `Lfu` found for enum `shared::eviction_policy::EvictionPolicyKind` in the current scope
   --> src\shared\traits\bucket.rs:317:59
    |
317 |         let mut bucket = TestBucket::new(2, EvictionPolicyKind::Lfu);
    |                                                                 ^^^ variant, associated function, or constant not found in `shared::eviction_policy::EvictionPolicyKind`
    |
   ::: src\shared\eviction_policy.rs:121:1
    |
121 | pub enum EvictionPolicyKind {
    | --------------------------- variant, associated function, or constant `Lfu` not found for this enum

error[E0599]: no variant, associated function, or constant named `Lfu` found for enum `shared::eviction_policy::EvictionPolicyKind` in the current scope
   --> src\shared\traits\bucket.rs:371:59
    |
371 |         let mut bucket = TestBucket::new(4, EvictionPolicyKind::Lfu);
    |                                                                 ^^^ variant, associated function, or constant not found in `shared::eviction_policy::EvictionPolicyKind`
    |
   ::: src\shared\eviction_policy.rs:121:1
    |
121 | pub enum EvictionPolicyKind {
    | --------------------------- variant, associated function, or constant `Lfu` not found for this enum

error[E0599]: no variant, associated function, or constant named `TimestampLRU` found for enum `shared::eviction_policy::EvictionPolicyKind` in the current scope
   --> src\shared\traits\bucket.rs:383:59
    |
383 |         let mut bucket = TestBucket::new(4, EvictionPolicyKind::TimestampLRU);
    |                                                                 ^^^^^^^^^^^^ variant, associated function, or constant not found in `shared::eviction_policy::EvictionPolicyKind`
    |
   ::: src\shared\eviction_policy.rs:121:1
    |
121 | pub enum EvictionPolicyKind {
    | --------------------------- variant, associated function, or constant `TimestampLRU` not found for this enum

Some errors have detailed explanations: E0599, E0609.
For more information about an error, try `rustc --explain E0599`.
error: could not compile `associative_cache` (lib test) due to 9 previous errors
warning: build failed, waiting for other jobs to finish...

```
---

## 2026-07-22T02:16:29Z — variant=cycle/0006/v001 hypothesis=H01 attempt=2

**Gate**: build gate
**Error**:
```
    Updating crates.io index
     Locking 86 packages to latest compatible versions
      Adding jni v0.20.0 (available: v0.22.4)
      Adding rand v0.8.7 (available: v0.10.2)
      Adding simplelog v0.11.2 (available: v0.12.2)
      Adding termcolor v1.1.3 (available: v1.4.1)
   Compiling windows-link v0.2.1
   Compiling cfg-if v1.0.4
   Compiling proc-macro2 v1.0.107
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.47
   Compiling getrandom v0.3.4
   Compiling zerocopy v0.8.55
   Compiling autocfg v1.5.1
   Compiling windows-sys v0.61.2
   Compiling getrandom v0.4.3
   Compiling getrandom v0.2.17
   Compiling jni-sys v0.3.1
   Compiling fastrand v2.5.0
   Compiling num-traits v0.2.19
   Compiling thiserror v1.0.69
   Compiling once_cell v1.21.4
   Compiling rand_core v0.9.5
   Compiling rand_core v0.6.4
   Compiling bytes v1.12.1
   Compiling bit-vec v0.8.0
   Compiling winapi-util v0.1.11
   Compiling syn v2.0.119
   Compiling same-file v1.0.6
   Compiling tempfile v3.27.0
   Compiling fnv v1.0.7
   Compiling quick-error v1.2.3
   Compiling walkdir v2.5.0
   Compiling log v0.4.33
   Compiling wait-timeout v0.2.1
   Compiling memchr v2.8.3
   Compiling rusty-fork v0.3.1
   Compiling termcolor v1.1.3
   Compiling jni v0.20.0
   Compiling chrono v0.4.45
   Compiling bit-set v0.8.0
   Compiling combine v4.6.7
   Compiling rand_xorshift v0.4.0
   Compiling rand v0.9.5
   Compiling regex-syntax v0.8.11
   Compiling cesu8 v1.1.0
   Compiling bitflags v2.13.1
   Compiling unarray v0.1.4
   Compiling simplelog v0.11.2
   Compiling framework v0.1.0 (C:\Users\yaniv\Projects\cache-evolve\framework-rs)
   Compiling jni-sys-macros v0.4.1
   Compiling ppv-lite86 v0.2.21
   Compiling thiserror-impl v1.0.69
   Compiling rand_chacha v0.9.0
   Compiling rand_chacha v0.3.1
   Compiling jni-sys v0.4.1
   Compiling proptest v1.11.0
   Compiling rand v0.8.7
   Compiling associative_cache v0.1.0 (C:\Users\yaniv\AppData\Local\Temp\variant-workspace-ii8_2ksa)
error[E0599]: no variant, associated function, or constant named `TimestampLRU` found for enum `EvictionPolicyKind` in the current scope
   --> tests\associative_cache_integration.rs:113:49
    |
113 |     let mut cache = tiny_cache(EvictionPolicyKind::TimestampLRU);
    |                                                    ^^^^^^^^^^^^ variant, associated function, or constant not found in `EvictionPolicyKind`

error[E0599]: no variant, associated function, or constant named `Lfu` found for enum `EvictionPolicyKind` in the current scope
   --> tests\associative_cache_integration.rs:128:49
    |
128 |     let mut cache = tiny_cache(EvictionPolicyKind::Lfu);
    |                                                    ^^^ variant, associated function, or constant not found in `EvictionPolicyKind`

For more information about this error, try `rustc --explain E0599`.
error: could not compile `associative_cache` (test "associative_cache_integration") due to 2 previous errors
warning: build failed, waiting for other jobs to finish...

```
---
