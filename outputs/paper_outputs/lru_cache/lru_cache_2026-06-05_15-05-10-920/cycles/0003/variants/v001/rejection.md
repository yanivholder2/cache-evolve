
## 2026-06-05T15:27:11Z — variant=cycle/0003/v001 hypothesis=H01 attempt=1

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
   Compiling getrandom v0.3.4
   Compiling zerocopy v0.8.50
   Compiling autocfg v1.5.1
   Compiling num-traits v0.2.19
   Compiling syn v2.0.117
   Compiling getrandom v0.4.2
   Compiling rustix v1.1.4
   Compiling rand_core v0.9.5
   Compiling linux-raw-sys v0.12.1
   Compiling same-file v1.0.6
   Compiling bitflags v2.12.1
   Compiling walkdir v2.5.0
   Compiling thiserror v1.0.69
   Compiling once_cell v1.21.4
   Compiling jni-sys v0.3.1
   Compiling fastrand v2.4.1
   Compiling tempfile v3.27.0
   Compiling jni v0.20.0
   Compiling ppv-lite86 v0.2.21
   Compiling getrandom v0.2.17
   Compiling wait-timeout v0.2.1
   Compiling bytes v1.11.1
   Compiling memchr v2.8.1
   Compiling quick-error v1.2.3
   Compiling log v0.4.32
   Compiling iana-time-zone v0.1.65
   Compiling fnv v1.0.7
   Compiling bit-vec v0.8.0
   Compiling rusty-fork v0.3.1
   Compiling jni-sys-macros v0.4.1
   Compiling thiserror-impl v1.0.69
   Compiling bit-set v0.8.0
   Compiling combine v4.6.7
   Compiling chrono v0.4.45
   Compiling jni-sys v0.4.1
   Compiling rand_chacha v0.9.0
   Compiling rand_core v0.6.4
   Compiling rand_xorshift v0.4.0
   Compiling rand v0.9.4
   Compiling cesu8 v1.1.0
   Compiling termcolor v1.1.3
   Compiling regex-syntax v0.8.10
   Compiling unarray v0.1.4
   Compiling simplelog v0.11.2
   Compiling rand_chacha v0.3.1
   Compiling framework v0.1.0 (/home/user/cache-evolve/framework-rs)
   Compiling rand v0.8.6
   Compiling proptest v1.11.0
   Compiling lru_cache v0.1.0 (/tmp/variant-workspace-o83wiss6)
error[E0382]: use of moved value: `value`
   --> src/lru_cache/mod.rs:149:27
    |
114 |     pub fn put(&mut self, key: K, value: V) {
    |                                   ----- move occurs because `value` has type `V`, which does not implement the `Copy` trait
...
138 |                 value,
    |                 ----- value moved here
...
149 |         self.nodes[idx].value = value;
    |                                 ^^^^^ value used here after move
    |
help: if `V` implemented `Clone`, you could clone the value
   --> src/lru_cache/mod.rs:50:9
    |
 50 | impl<K, V> LruCache<K, V>
    |         ^ consider constraining this type parameter with `Clone`
...
138 |                 value,
    |                 ----- you could clone this value

For more information about this error, try `rustc --explain E0382`.
error: could not compile `lru_cache` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `lru_cache` (lib test) due to 1 previous error

```
---
