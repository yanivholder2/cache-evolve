
## 2026-07-19T16:54:03Z — variant=cycle/0002/v003 hypothesis=H03 attempt=1

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
   Compiling once_cell v1.21.4
   Compiling num-traits v0.2.19
   Compiling fastrand v2.5.0
   Compiling thiserror v1.0.69
   Compiling rand_core v0.9.5
   Compiling jni-sys v0.3.1
   Compiling log v0.4.33
   Compiling winapi-util v0.1.11
   Compiling tempfile v3.27.0
   Compiling same-file v1.0.6
   Compiling fnv v1.0.7
   Compiling walkdir v2.5.0
   Compiling syn v2.0.119
   Compiling wait-timeout v0.2.1
   Compiling bytes v1.12.1
   Compiling bit-vec v0.8.0
   Compiling quick-error v1.2.3
   Compiling memchr v2.8.3
   Compiling rusty-fork v0.3.1
   Compiling jni v0.20.0
   Compiling chrono v0.4.45
   Compiling bit-set v0.8.0
   Compiling combine v4.6.7
   Compiling termcolor v1.1.3
   Compiling rand_xorshift v0.4.0
   Compiling rand v0.9.5
   Compiling regex-syntax v0.8.11
   Compiling unarray v0.1.4
   Compiling cesu8 v1.1.0
   Compiling bitflags v2.13.1
   Compiling simplelog v0.11.2
   Compiling getrandom v0.2.17
   Compiling framework v0.1.0 (C:\Users\yaniv\Projects\cache-evolve\framework-rs)
   Compiling rand_core v0.6.4
   Compiling ppv-lite86 v0.2.21
   Compiling rand_chacha v0.9.0
   Compiling rand_chacha v0.3.1
   Compiling rand v0.8.7
   Compiling jni-sys-macros v0.4.1
   Compiling thiserror-impl v1.0.69
   Compiling proptest v1.11.0
   Compiling jni-sys v0.4.1
   Compiling lru_cache v0.1.0 (C:\Users\yaniv\AppData\Local\Temp\variant-workspace-j522_ill)
error[E0499]: cannot borrow `self.map` as mutable more than once at a time
   --> src\lru_cache\mod.rs:101:6
    |
 84 |         match self.map.entry(key) {
    |               -------- first mutable borrow occurs here
...
101 |                     self.map.remove(&evicted_key);
    |                     ^^^^^^^^ second mutable borrow occurs here
102 |                     self.unlink(head_idx);
103 |                     self.nodes[head_idx].key = e.key().clone();
    |                                                - first borrow later used here

error[E0499]: cannot borrow `*self` as mutable more than once at a time
   --> src\lru_cache\mod.rs:102:6
    |
 84 |         match self.map.entry(key) {
    |               -------- first mutable borrow occurs here
...
102 |                     self.unlink(head_idx);
    |                     ^^^^ second mutable borrow occurs here
103 |                     self.nodes[head_idx].key = e.key().clone();
    |                                                - first borrow later used here

error[E0499]: cannot borrow `*self` as mutable more than once at a time
   --> src\lru_cache\mod.rs:118:5
    |
 84 |         match self.map.entry(key) {
    |               -------- first mutable borrow occurs here
...
118 |                 self.push_back(idx);
    |                 ^^^^ second mutable borrow occurs here
119 |                 e.insert(idx);
    |                 - first borrow later used here

For more information about this error, try `rustc --explain E0499`.
error: could not compile `lru_cache` (lib) due to 3 previous errors
warning: build failed, waiting for other jobs to finish...
error: could not compile `lru_cache` (lib test) due to 3 previous errors

```
---
