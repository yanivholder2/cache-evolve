
## 2026-07-21T11:42:24Z — variant=cycle/0006/v001 hypothesis=H01 attempt=1

**Gate**: build gate
**Error**:
```
    Updating crates.io index
    Blocking waiting for file lock on package cache
     Locking 86 packages to latest compatible versions
      Adding jni v0.20.0 (available: v0.22.4)
      Adding rand v0.8.7 (available: v0.10.2)
      Adding simplelog v0.11.2 (available: v0.12.2)
      Adding termcolor v1.1.3 (available: v1.4.1)
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
   Compiling windows-link v0.2.1
   Compiling cfg-if v1.0.4
   Compiling proc-macro2 v1.0.107
   Compiling quote v1.0.47
   Compiling unicode-ident v1.0.24
   Compiling zerocopy v0.8.55
   Compiling getrandom v0.3.4
   Compiling autocfg v1.5.1
   Compiling windows-sys v0.61.2
   Compiling getrandom v0.4.3
   Compiling getrandom v0.2.17
   Compiling once_cell v1.21.4
   Compiling num-traits v0.2.19
   Compiling thiserror v1.0.69
   Compiling jni-sys v0.3.1
   Compiling rand_core v0.9.5
   Compiling fastrand v2.5.0
   Compiling winapi-util v0.1.11
   Compiling tempfile v3.27.0
   Compiling rand_core v0.6.4
   Compiling syn v2.0.119
   Compiling same-file v1.0.6
   Compiling log v0.4.33
   Compiling bytes v1.12.1
   Compiling walkdir v2.5.0
   Compiling quick-error v1.2.3
   Compiling bit-vec v0.8.0
   Compiling fnv v1.0.7
   Compiling wait-timeout v0.2.1
   Compiling memchr v2.8.3
   Compiling bit-set v0.8.0
   Compiling jni v0.20.0
   Compiling chrono v0.4.45
   Compiling rusty-fork v0.3.1
   Compiling termcolor v1.1.3
   Compiling rand_xorshift v0.4.0
   Compiling combine v4.6.7
   Compiling rand v0.9.5
   Compiling cesu8 v1.1.0
   Compiling unarray v0.1.4
   Compiling regex-syntax v0.8.11
   Compiling bitflags v2.13.1
   Compiling simplelog v0.11.2
   Compiling framework v0.1.0 (C:\Users\yaniv\Projects\cache-evolve\framework-rs)
   Compiling ppv-lite86 v0.2.21
   Compiling jni-sys-macros v0.4.1
   Compiling thiserror-impl v1.0.69
   Compiling rand_chacha v0.9.0
   Compiling rand_chacha v0.3.1
   Compiling rand v0.8.7
   Compiling proptest v1.11.0
   Compiling jni-sys v0.4.1
   Compiling associative_cache v0.1.0 (C:\Users\yaniv\AppData\Local\Temp\variant-workspace-4s13mjz6)
error[E0503]: cannot use `self.epoch` because it was mutably borrowed
  --> src\associative_cache\mod.rs:63:28
   |
61 |         let bucket = self.get_mut_bucket(&key);
   |                      ---- `*self` is borrowed here
62 |         // Set the current epoch on the bucket before put, so eviction (if needed) has access to it
63 |         bucket.set_current_epoch(self.epoch);
   |                                  ^^^^^^^^^^ use of borrowed `*self`
...
68 |         bucket.put(item);
   |         ------ borrow later used here

error[E0503]: cannot use `self.epoch` because it was mutably borrowed
  --> src\associative_cache\mod.rs:67:28
   |
61 |         let bucket = self.get_mut_bucket(&key);
   |                      ---- `*self` is borrowed here
...
67 |         item.last_access_epoch = self.epoch;
   |                                  ^^^^^^^^^^ use of borrowed `*self`
68 |         bucket.put(item);
   |         ------ borrow later used here

For more information about this error, try `rustc --explain E0503`.
error: could not compile `associative_cache` (lib) due to 2 previous errors
warning: build failed, waiting for other jobs to finish...
error: could not compile `associative_cache` (lib test) due to 2 previous errors

```
---
