
## 2026-06-10T20:48:20Z — variant=cycle/0003/v003 hypothesis=H03 attempt=1

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
   Compiling zerocopy v0.8.52
   Compiling getrandom v0.3.4
   Compiling autocfg v1.5.1
   Compiling syn v2.0.117
   Compiling num-traits v0.2.19
   Compiling rustix v1.1.4
   Compiling getrandom v0.4.2
   Compiling rand_core v0.9.5
   Compiling bitflags v2.13.0
   Compiling same-file v1.0.6
   Compiling linux-raw-sys v0.12.1
   Compiling walkdir v2.5.0
   Compiling getrandom v0.2.17
   Compiling jni-sys v0.3.1
   Compiling thiserror v1.0.69
   Compiling once_cell v1.21.4
   Compiling fastrand v2.4.1
   Compiling tempfile v3.27.0
   Compiling ppv-lite86 v0.2.21
   Compiling rand_core v0.6.4
   Compiling jni v0.20.0
   Compiling wait-timeout v0.2.1
   Compiling iana-time-zone v0.1.65
   Compiling bytes v1.11.1
   Compiling bit-vec v0.8.0
   Compiling quick-error v1.2.3
   Compiling jni-sys-macros v0.4.1
   Compiling thiserror-impl v1.0.69
   Compiling fnv v1.0.7
   Compiling memchr v2.8.1
   Compiling log v0.4.32
   Compiling jni-sys v0.4.1
   Compiling rusty-fork v0.3.1
   Compiling combine v4.6.7
   Compiling bit-set v0.8.0
   Compiling chrono v0.4.45
   Compiling rand_chacha v0.3.1
   Compiling rand_chacha v0.9.0
   Compiling rand_xorshift v0.4.0
   Compiling rand v0.9.4
   Compiling cesu8 v1.1.0
   Compiling unarray v0.1.4
   Compiling regex-syntax v0.8.11
   Compiling termcolor v1.1.3
   Compiling simplelog v0.11.2
   Compiling rand v0.8.6
   Compiling framework v0.1.0 (/home/user/cache-evolve/framework-rs)
   Compiling proptest v1.11.0
   Compiling associative_cache v0.1.0 (/tmp/variant-workspace-h462etpd)
error[E0502]: cannot borrow `*self` as immutable because it is also borrowed as mutable
  --> src/associative_cache/mod.rs:96:3
   |
78 |     pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
   |                                - let's call the lifetime of this reference `'1`
...
83 |             let bucket = &mut self.buckets[bucket_index];
   |                               ------------ mutable borrow occurs here
...
88 |                 return Some(&item.value);
   |                        ----------------- returning this value requires that `self.buckets` is borrowed for `'1`
...
96 |         self.get_from_stash(key)
   |         ^^^^ immutable borrow occurs here

For more information about this error, try `rustc --explain E0502`.
error: could not compile `associative_cache` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `associative_cache` (lib test) due to 1 previous error

```
---

## 2026-06-10T20:50:29Z — variant=cycle/0003/v003 hypothesis=H03 attempt=2

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
   Compiling zerocopy v0.8.52
   Compiling autocfg v1.5.1
   Compiling num-traits v0.2.19
   Compiling getrandom v0.4.2
   Compiling rustix v1.1.4
   Compiling syn v2.0.117
   Compiling rand_core v0.9.5
   Compiling linux-raw-sys v0.12.1
   Compiling same-file v1.0.6
   Compiling bitflags v2.13.0
   Compiling walkdir v2.5.0
   Compiling getrandom v0.2.17
   Compiling jni-sys v0.3.1
   Compiling thiserror v1.0.69
   Compiling fastrand v2.4.1
   Compiling once_cell v1.21.4
   Compiling tempfile v3.27.0
   Compiling rand_core v0.6.4
   Compiling jni v0.20.0
   Compiling wait-timeout v0.2.1
   Compiling ppv-lite86 v0.2.21
   Compiling iana-time-zone v0.1.65
   Compiling quick-error v1.2.3
   Compiling bytes v1.11.1
   Compiling fnv v1.0.7
   Compiling bit-vec v0.8.0
   Compiling memchr v2.8.1
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
   Compiling rand v0.9.4
   Compiling rand_xorshift v0.4.0
   Compiling regex-syntax v0.8.11
   Compiling termcolor v1.1.3
   Compiling unarray v0.1.4
   Compiling cesu8 v1.1.0
   Compiling simplelog v0.11.2
   Compiling rand v0.8.6
   Compiling framework v0.1.0 (/home/user/cache-evolve/framework-rs)
   Compiling proptest v1.11.0
   Compiling associative_cache v0.1.0 (/tmp/variant-workspace-6gni0w6c)
error[E0502]: cannot borrow `*self` as immutable because it is also borrowed as mutable
  --> src/associative_cache/mod.rs:97:5
   |
78 |     pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
   |                                - let's call the lifetime of this reference `'1`
...
83 |             let bucket = &mut self.buckets[bucket_index];
   |                               ------------ mutable borrow occurs here
...
90 |                 Some(&item.value)
   |                 ----------------- returning this value requires that `self.buckets` is borrowed for `'1`
...
97 |                 self.get_from_stash(key)
   |                 ^^^^ immutable borrow occurs here

For more information about this error, try `rustc --explain E0502`.
error: could not compile `associative_cache` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `associative_cache` (lib test) due to 1 previous error

```
---

## 2026-06-10T20:52:40Z — variant=cycle/0003/v003 hypothesis=H03 attempt=3

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
   Compiling quote v1.0.45
   Compiling unicode-ident v1.0.24
   Compiling zerocopy v0.8.52
   Compiling getrandom v0.3.4
   Compiling autocfg v1.5.1
   Compiling syn v2.0.117
   Compiling num-traits v0.2.19
   Compiling rustix v1.1.4
   Compiling getrandom v0.4.2
   Compiling rand_core v0.9.5
   Compiling linux-raw-sys v0.12.1
   Compiling same-file v1.0.6
   Compiling bitflags v2.13.0
   Compiling walkdir v2.5.0
   Compiling getrandom v0.2.17
   Compiling fastrand v2.4.1
   Compiling once_cell v1.21.4
   Compiling jni-sys v0.3.1
   Compiling thiserror v1.0.69
   Compiling tempfile v3.27.0
   Compiling rand_core v0.6.4
   Compiling ppv-lite86 v0.2.21
   Compiling jni v0.20.0
   Compiling wait-timeout v0.2.1
   Compiling bytes v1.11.1
   Compiling log v0.4.32
   Compiling jni-sys-macros v0.4.1
   Compiling thiserror-impl v1.0.69
   Compiling memchr v2.8.1
   Compiling jni-sys v0.4.1
   Compiling bit-vec v0.8.0
   Compiling fnv v1.0.7
   Compiling iana-time-zone v0.1.65
   Compiling quick-error v1.2.3
   Compiling chrono v0.4.45
   Compiling rusty-fork v0.3.1
   Compiling bit-set v0.8.0
   Compiling combine v4.6.7
   Compiling rand_chacha v0.3.1
   Compiling rand_chacha v0.9.0
   Compiling rand_xorshift v0.4.0
   Compiling rand v0.9.4
   Compiling cesu8 v1.1.0
   Compiling unarray v0.1.4
   Compiling regex-syntax v0.8.11
   Compiling termcolor v1.1.3
   Compiling simplelog v0.11.2
   Compiling rand v0.8.6
   Compiling framework v0.1.0 (/home/user/cache-evolve/framework-rs)
   Compiling proptest v1.11.0
   Compiling associative_cache v0.1.0 (/tmp/variant-workspace-f6fgeuob)
error[E0502]: cannot borrow `*self` as immutable because it is also borrowed as mutable
  --> src/associative_cache/mod.rs:96:3
   |
78 |     pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
   |                                - let's call the lifetime of this reference `'1`
...
83 |             let bucket = &mut self.buckets[bucket_index];
   |                               ------------ mutable borrow occurs here
...
88 |                 return Some(&item.value);
   |                        ----------------- returning this value requires that `self.buckets` is borrowed for `'1`
...
96 |         self.get_from_stash(key)
   |         ^^^^ immutable borrow occurs here

For more information about this error, try `rustc --explain E0502`.
error: could not compile `associative_cache` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `associative_cache` (lib test) due to 1 previous error

```
---
