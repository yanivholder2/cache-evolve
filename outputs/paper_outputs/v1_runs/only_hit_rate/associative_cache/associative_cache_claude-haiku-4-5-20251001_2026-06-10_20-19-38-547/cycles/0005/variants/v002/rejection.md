
## 2026-06-10T21:12:49Z — variant=cycle/0005/v002 hypothesis=H02 attempt=1

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
   Compiling getrandom v0.4.2
   Compiling rustix v1.1.4
   Compiling rand_core v0.9.5
   Compiling linux-raw-sys v0.12.1
   Compiling bitflags v2.13.0
   Compiling same-file v1.0.6
   Compiling walkdir v2.5.0
   Compiling getrandom v0.2.17
   Compiling thiserror v1.0.69
   Compiling fastrand v2.4.1
   Compiling jni-sys v0.3.1
   Compiling ppv-lite86 v0.2.21
   Compiling once_cell v1.21.4
   Compiling tempfile v3.27.0
   Compiling rand_core v0.6.4
   Compiling jni-sys-macros v0.4.1
   Compiling thiserror-impl v1.0.69
   Compiling jni v0.20.0
   Compiling wait-timeout v0.2.1
   Compiling jni-sys v0.4.1
   Compiling memchr v2.8.1
   Compiling log v0.4.32
   Compiling quick-error v1.2.3
   Compiling bytes v1.11.1
   Compiling bit-vec v0.8.0
   Compiling iana-time-zone v0.1.65
   Compiling fnv v1.0.7
   Compiling combine v4.6.7
   Compiling rusty-fork v0.3.1
   Compiling chrono v0.4.45
   Compiling bit-set v0.8.0
   Compiling rand_chacha v0.3.1
   Compiling rand_chacha v0.9.0
   Compiling rand v0.9.4
   Compiling rand_xorshift v0.4.0
   Compiling termcolor v1.1.3
   Compiling regex-syntax v0.8.11
   Compiling cesu8 v1.1.0
   Compiling unarray v0.1.4
   Compiling simplelog v0.11.2
   Compiling rand v0.8.6
   Compiling framework v0.1.0 (/home/user/cache-evolve/framework-rs)
   Compiling proptest v1.11.0
   Compiling associative_cache v0.1.0 (/tmp/variant-workspace-9y7xzuod)
error[E0499]: cannot borrow `self.buckets` as mutable more than once at a time
   --> src/associative_cache/mod.rs:106:22
    |
 85 |     pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
    |                                - let's call the lifetime of this reference `'1`
...
 90 |             let bucket = &mut self.buckets[bucket_index];
    |                               ------------ first mutable borrow occurs here
...
 97 |                 return Some(&item.value);
    |                        ----------------- returning this value requires that `self.buckets` is borrowed for `'1`
...
106 |             let bucket = &mut self.buckets[bucket_index];
    |                               ^^^^^^^^^^^^ second mutable borrow occurs here

For more information about this error, try `rustc --explain E0499`.
error: could not compile `associative_cache` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `associative_cache` (lib test) due to 1 previous error

```
---

## 2026-06-10T21:17:18Z — variant=cycle/0005/v002 hypothesis=H02 attempt=2

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
   Compiling getrandom v0.4.2
   Compiling rustix v1.1.4
   Compiling rand_core v0.9.5
   Compiling linux-raw-sys v0.12.1
   Compiling same-file v1.0.6
   Compiling bitflags v2.13.0
   Compiling walkdir v2.5.0
   Compiling getrandom v0.2.17
   Compiling fastrand v2.4.1
   Compiling jni-sys v0.3.1
   Compiling thiserror v1.0.69
   Compiling once_cell v1.21.4
   Compiling tempfile v3.27.0
   Compiling rand_core v0.6.4
   Compiling ppv-lite86 v0.2.21
   Compiling jni v0.20.0
   Compiling wait-timeout v0.2.1
   Compiling iana-time-zone v0.1.65
   Compiling log v0.4.32
   Compiling fnv v1.0.7
   Compiling bit-vec v0.8.0
   Compiling jni-sys-macros v0.4.1
   Compiling thiserror-impl v1.0.69
   Compiling bytes v1.11.1
   Compiling quick-error v1.2.3
   Compiling memchr v2.8.1
   Compiling jni-sys v0.4.1
   Compiling combine v4.6.7
   Compiling rusty-fork v0.3.1
   Compiling bit-set v0.8.0
   Compiling chrono v0.4.45
   Compiling rand_chacha v0.3.1
   Compiling rand_chacha v0.9.0
   Compiling rand_xorshift v0.4.0
   Compiling rand v0.9.4
   Compiling unarray v0.1.4
   Compiling regex-syntax v0.8.11
   Compiling cesu8 v1.1.0
   Compiling termcolor v1.1.3
   Compiling simplelog v0.11.2
   Compiling rand v0.8.6
   Compiling framework v0.1.0 (/home/user/cache-evolve/framework-rs)
   Compiling proptest v1.11.0
   Compiling associative_cache v0.1.0 (/tmp/variant-workspace-hb0n_ffr)
error[E0499]: cannot borrow `self.buckets` as mutable more than once at a time
   --> src/associative_cache/mod.rs:105:22
    |
 85 |     pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
    |                                - let's call the lifetime of this reference `'1`
...
 90 |             let bucket = &mut self.buckets[bucket_index];
    |                               ------------ first mutable borrow occurs here
 91 |             if let Some(item) = bucket.get(key) {
 92 |                 return Some(&item.value);
    |                        ----------------- returning this value requires that `self.buckets` is borrowed for `'1`
...
105 |             let bucket = &mut self.buckets[bucket_index];
    |                               ^^^^^^^^^^^^ second mutable borrow occurs here

For more information about this error, try `rustc --explain E0499`.
error: could not compile `associative_cache` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `associative_cache` (lib test) due to 1 previous error

```
---

## 2026-06-10T21:20:27Z — variant=cycle/0005/v002 hypothesis=H02 attempt=3

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
   Compiling getrandom v0.3.4
   Compiling zerocopy v0.8.52
   Compiling autocfg v1.5.1
   Compiling syn v2.0.117
   Compiling num-traits v0.2.19
   Compiling getrandom v0.4.2
   Compiling rustix v1.1.4
   Compiling rand_core v0.9.5
   Compiling linux-raw-sys v0.12.1
   Compiling bitflags v2.13.0
   Compiling same-file v1.0.6
   Compiling walkdir v2.5.0
   Compiling getrandom v0.2.17
   Compiling thiserror v1.0.69
   Compiling once_cell v1.21.4
   Compiling jni-sys v0.3.1
   Compiling fastrand v2.4.1
   Compiling tempfile v3.27.0
   Compiling ppv-lite86 v0.2.21
   Compiling rand_core v0.6.4
   Compiling jni v0.20.0
   Compiling wait-timeout v0.2.1
   Compiling jni-sys-macros v0.4.1
   Compiling thiserror-impl v1.0.69
   Compiling memchr v2.8.1
   Compiling bytes v1.11.1
   Compiling jni-sys v0.4.1
   Compiling log v0.4.32
   Compiling quick-error v1.2.3
   Compiling bit-vec v0.8.0
   Compiling iana-time-zone v0.1.65
   Compiling fnv v1.0.7
   Compiling rusty-fork v0.3.1
   Compiling bit-set v0.8.0
   Compiling chrono v0.4.45
   Compiling combine v4.6.7
   Compiling rand_chacha v0.9.0
   Compiling rand_chacha v0.3.1
   Compiling rand_xorshift v0.4.0
   Compiling rand v0.9.4
   Compiling regex-syntax v0.8.11
   Compiling unarray v0.1.4
   Compiling termcolor v1.1.3
   Compiling cesu8 v1.1.0
   Compiling simplelog v0.11.2
   Compiling rand v0.8.6
   Compiling framework v0.1.0 (/home/user/cache-evolve/framework-rs)
   Compiling proptest v1.11.0
   Compiling associative_cache v0.1.0 (/tmp/variant-workspace-86duqiy3)
error[E0499]: cannot borrow `*bucket` as mutable more than once at a time
   --> src/associative_cache/mod.rs:106:36
    |
 85 |     pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
    |                                - let's call the lifetime of this reference `'1`
...
 93 |             if let Some(item) = bucket.get(key) {
    |                                 ------ first mutable borrow occurs here
 94 |                 return Some(&item.value);
    |                        ----------------- returning this value requires that `*bucket` is borrowed for `'1`
...
106 |                 let (inserted_item, evicted) = bucket.put(stashed_item);
    |                                                ^^^^^^ second mutable borrow occurs here

For more information about this error, try `rustc --explain E0499`.
error: could not compile `associative_cache` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `associative_cache` (lib test) due to 1 previous error

```
---
