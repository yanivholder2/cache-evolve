
## 2026-07-04T20:59:11Z — variant=cycle/0010/v002 hypothesis=H02 attempt=1

**Gate**: build gate
**Error**:
```
    Updating crates.io index
     Locking 86 packages to latest compatible versions
      Adding jni v0.20.0 (available: v0.22.4)
      Adding rand v0.8.6 (available: v0.10.2)
      Adding simplelog v0.11.2 (available: v0.12.2)
      Adding termcolor v1.1.3 (available: v1.4.1)
   Compiling libc v0.2.186
   Compiling cfg-if v1.0.4
   Compiling proc-macro2 v1.0.106
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.46
   Compiling getrandom v0.3.4
   Compiling zerocopy v0.8.52
   Compiling autocfg v1.5.1
   Compiling num-traits v0.2.19
   Compiling rustix v1.1.4
   Compiling getrandom v0.4.3
   Compiling syn v2.0.118
   Compiling rand_core v0.9.5
   Compiling bitflags v2.13.0
   Compiling linux-raw-sys v0.12.1
   Compiling same-file v1.0.6
   Compiling walkdir v2.5.0
   Compiling getrandom v0.2.17
   Compiling once_cell v1.21.4
   Compiling thiserror v1.0.69
   Compiling fastrand v2.4.1
   Compiling jni-sys v0.3.1
   Compiling tempfile v3.27.0
   Compiling ppv-lite86 v0.2.21
   Compiling rand_core v0.6.4
   Compiling jni v0.20.0
   Compiling wait-timeout v0.2.1
   Compiling bytes v1.12.0
   Compiling fnv v1.0.7
   Compiling log v0.4.33
   Compiling bit-vec v0.8.0
   Compiling quick-error v1.2.3
   Compiling memchr v2.8.2
   Compiling iana-time-zone v0.1.65
   Compiling chrono v0.4.45
   Compiling jni-sys-macros v0.4.1
   Compiling thiserror-impl v1.0.69
   Compiling jni-sys v0.4.1
   Compiling combine v4.6.7
   Compiling bit-set v0.8.0
   Compiling rusty-fork v0.3.1
   Compiling rand_chacha v0.9.0
   Compiling rand_chacha v0.3.1
   Compiling rand_xorshift v0.4.0
   Compiling rand v0.9.4
   Compiling regex-syntax v0.8.11
   Compiling cesu8 v1.1.0
   Compiling unarray v0.1.4
   Compiling termcolor v1.1.3
   Compiling rand v0.8.6
   Compiling simplelog v0.11.2
   Compiling framework v0.1.0 (/home/user/cache-evolve/framework-rs)
   Compiling proptest v1.11.0
   Compiling associative_cache v0.1.0 (/tmp/variant-workspace-tngr2tor)
error[E0502]: cannot borrow `*self` as immutable because it is also borrowed as mutable
   --> src/associative_cache/mod.rs:110:24
    |
 98 |     pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
    |                                - let's call the lifetime of this reference `'1`
 99 |         let bucket_index = get_index(key, self.buckets.len());
100 |         let bucket = &mut self.buckets[bucket_index];
    |                           ------------ mutable borrow occurs here
...
106 |             return Some(&data?.value);
    |                    ------------------ returning this value requires that `self.buckets` is borrowed for `'1`
...
110 |         if let Some(index) = self.find_in_stash(&self.hot_stash, key) {
    |                              ^^^^ immutable borrow occurs here

error[E0502]: cannot borrow `*self` as immutable because it is also borrowed as mutable
   --> src/associative_cache/mod.rs:119:24
    |
 98 |     pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
    |                                - let's call the lifetime of this reference `'1`
 99 |         let bucket_index = get_index(key, self.buckets.len());
100 |         let bucket = &mut self.buckets[bucket_index];
    |                           ------------ mutable borrow occurs here
...
106 |             return Some(&data?.value);
    |                    ------------------ returning this value requires that `self.buckets` is borrowed for `'1`
...
119 |         if let Some(index) = self.find_in_stash(&self.warm_stash, key) {
    |                              ^^^^ immutable borrow occurs here

error[E0499]: cannot borrow `*self` as mutable more than once at a time
   --> src/associative_cache/mod.rs:123:4
    |
 98 |     pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
    |                                - let's call the lifetime of this reference `'1`
 99 |         let bucket_index = get_index(key, self.buckets.len());
100 |         let bucket = &mut self.buckets[bucket_index];
    |                           ------------ first mutable borrow occurs here
...
106 |             return Some(&data?.value);
    |                    ------------------ returning this value requires that `self.buckets` is borrowed for `'1`
...
123 |             self.promote_warm_to_hot(item);
    |             ^^^^ second mutable borrow occurs here

error[E0502]: cannot borrow `*self` as immutable because it is also borrowed as mutable
   --> src/associative_cache/mod.rs:128:24
    |
 98 |     pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
    |                                - let's call the lifetime of this reference `'1`
 99 |         let bucket_index = get_index(key, self.buckets.len());
100 |         let bucket = &mut self.buckets[bucket_index];
    |                           ------------ mutable borrow occurs here
...
106 |             return Some(&data?.value);
    |                    ------------------ returning this value requires that `self.buckets` is borrowed for `'1`
...
128 |         if let Some(index) = self.find_in_stash(&self.cold_stash, key) {
    |                              ^^^^ immutable borrow occurs here

error[E0499]: cannot borrow `*self` as mutable more than once at a time
   --> src/associative_cache/mod.rs:132:4
    |
 98 |     pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
    |                                - let's call the lifetime of this reference `'1`
 99 |         let bucket_index = get_index(key, self.buckets.len());
100 |         let bucket = &mut self.buckets[bucket_index];
    |                           ------------ first mutable borrow occurs here
...
106 |             return Some(&data?.value);
    |                    ------------------ returning this value requires that `self.buckets` is borrowed for `'1`
...
132 |             self.promote_cold_to_warm(item);
    |             ^^^^ second mutable borrow occurs here

Some errors have detailed explanations: E0499, E0502.
For more information about an error, try `rustc --explain E0499`.
error: could not compile `associative_cache` (lib) due to 5 previous errors
warning: build failed, waiting for other jobs to finish...
error: could not compile `associative_cache` (lib test) due to 5 previous errors

```
---

## 2026-07-04T21:00:59Z — variant=cycle/0010/v002 hypothesis=H02 attempt=2

**Gate**: build gate
**Error**:
```
    Updating crates.io index
     Locking 86 packages to latest compatible versions
      Adding jni v0.20.0 (available: v0.22.4)
      Adding rand v0.8.6 (available: v0.10.2)
      Adding simplelog v0.11.2 (available: v0.12.2)
      Adding termcolor v1.1.3 (available: v1.4.1)
   Compiling libc v0.2.186
   Compiling cfg-if v1.0.4
   Compiling proc-macro2 v1.0.106
   Compiling quote v1.0.46
   Compiling unicode-ident v1.0.24
   Compiling zerocopy v0.8.52
   Compiling getrandom v0.3.4
   Compiling autocfg v1.5.1
   Compiling syn v2.0.118
   Compiling num-traits v0.2.19
   Compiling getrandom v0.4.3
   Compiling rustix v1.1.4
   Compiling rand_core v0.9.5
   Compiling same-file v1.0.6
   Compiling bitflags v2.13.0
   Compiling linux-raw-sys v0.12.1
   Compiling walkdir v2.5.0
   Compiling getrandom v0.2.17
   Compiling jni-sys v0.3.1
   Compiling thiserror v1.0.69
   Compiling once_cell v1.21.4
   Compiling fastrand v2.4.1
   Compiling ppv-lite86 v0.2.21
   Compiling tempfile v3.27.0
   Compiling rand_core v0.6.4
   Compiling jni v0.20.0
   Compiling wait-timeout v0.2.1
   Compiling memchr v2.8.2
   Compiling log v0.4.33
   Compiling bytes v1.12.0
   Compiling fnv v1.0.7
   Compiling quick-error v1.2.3
   Compiling iana-time-zone v0.1.65
   Compiling jni-sys-macros v0.4.1
   Compiling thiserror-impl v1.0.69
   Compiling bit-vec v0.8.0
   Compiling combine v4.6.7
   Compiling jni-sys v0.4.1
   Compiling bit-set v0.8.0
   Compiling chrono v0.4.45
   Compiling rusty-fork v0.3.1
   Compiling rand_chacha v0.3.1
   Compiling rand_chacha v0.9.0
   Compiling rand v0.9.4
   Compiling rand_xorshift v0.4.0
   Compiling unarray v0.1.4
   Compiling cesu8 v1.1.0
   Compiling regex-syntax v0.8.11
   Compiling termcolor v1.1.3
   Compiling simplelog v0.11.2
   Compiling rand v0.8.6
   Compiling framework v0.1.0 (/home/user/cache-evolve/framework-rs)
   Compiling proptest v1.11.0
   Compiling associative_cache v0.1.0 (/tmp/variant-workspace-6le_kjea)
error[E0502]: cannot borrow `*self` as immutable because it is also borrowed as mutable
   --> src/associative_cache/mod.rs:114:24
    |
 98 |     pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
    |                                - let's call the lifetime of this reference `'1`
...
103 |             let bucket = &mut self.buckets[bucket_index];
    |                               ------------ mutable borrow occurs here
...
109 |                 return Some(&data?.value);
    |                        ------------------ returning this value requires that `self.buckets` is borrowed for `'1`
...
114 |         if let Some(index) = self.find_in_stash(&self.hot_stash, key) {
    |                              ^^^^ immutable borrow occurs here

error[E0502]: cannot borrow `*self` as immutable because it is also borrowed as mutable
   --> src/associative_cache/mod.rs:123:24
    |
 98 |     pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
    |                                - let's call the lifetime of this reference `'1`
...
103 |             let bucket = &mut self.buckets[bucket_index];
    |                               ------------ mutable borrow occurs here
...
109 |                 return Some(&data?.value);
    |                        ------------------ returning this value requires that `self.buckets` is borrowed for `'1`
...
123 |         if let Some(index) = self.find_in_stash(&self.warm_stash, key) {
    |                              ^^^^ immutable borrow occurs here

error[E0499]: cannot borrow `*self` as mutable more than once at a time
   --> src/associative_cache/mod.rs:127:4
    |
 98 |     pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
    |                                - let's call the lifetime of this reference `'1`
...
103 |             let bucket = &mut self.buckets[bucket_index];
    |                               ------------ first mutable borrow occurs here
...
109 |                 return Some(&data?.value);
    |                        ------------------ returning this value requires that `self.buckets` is borrowed for `'1`
...
127 |             self.promote_warm_to_hot(item);
    |             ^^^^ second mutable borrow occurs here

error[E0502]: cannot borrow `*self` as immutable because it is also borrowed as mutable
   --> src/associative_cache/mod.rs:132:24
    |
 98 |     pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
    |                                - let's call the lifetime of this reference `'1`
...
103 |             let bucket = &mut self.buckets[bucket_index];
    |                               ------------ mutable borrow occurs here
...
109 |                 return Some(&data?.value);
    |                        ------------------ returning this value requires that `self.buckets` is borrowed for `'1`
...
132 |         if let Some(index) = self.find_in_stash(&self.cold_stash, key) {
    |                              ^^^^ immutable borrow occurs here

error[E0499]: cannot borrow `*self` as mutable more than once at a time
   --> src/associative_cache/mod.rs:136:4
    |
 98 |     pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
    |                                - let's call the lifetime of this reference `'1`
...
103 |             let bucket = &mut self.buckets[bucket_index];
    |                               ------------ first mutable borrow occurs here
...
109 |                 return Some(&data?.value);
    |                        ------------------ returning this value requires that `self.buckets` is borrowed for `'1`
...
136 |             self.promote_cold_to_warm(item);
    |             ^^^^ second mutable borrow occurs here

Some errors have detailed explanations: E0499, E0502.
For more information about an error, try `rustc --explain E0499`.
error: could not compile `associative_cache` (lib) due to 5 previous errors
warning: build failed, waiting for other jobs to finish...
error: could not compile `associative_cache` (lib test) due to 5 previous errors

```
---

## 2026-07-04T21:04:58Z — variant=cycle/0010/v002 hypothesis=H02 attempt=3

**Gate**: build gate
**Error**:
```
    Updating crates.io index
     Locking 86 packages to latest compatible versions
      Adding jni v0.20.0 (available: v0.22.4)
      Adding rand v0.8.6 (available: v0.10.2)
      Adding simplelog v0.11.2 (available: v0.12.2)
      Adding termcolor v1.1.3 (available: v1.4.1)
   Compiling libc v0.2.186
   Compiling cfg-if v1.0.4
   Compiling proc-macro2 v1.0.106
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.46
   Compiling getrandom v0.3.4
   Compiling zerocopy v0.8.52
   Compiling autocfg v1.5.1
   Compiling num-traits v0.2.19
   Compiling syn v2.0.118
   Compiling getrandom v0.4.3
   Compiling rustix v1.1.4
   Compiling rand_core v0.9.5
   Compiling linux-raw-sys v0.12.1
   Compiling bitflags v2.13.0
   Compiling same-file v1.0.6
   Compiling walkdir v2.5.0
   Compiling getrandom v0.2.17
   Compiling fastrand v2.4.1
   Compiling thiserror v1.0.69
   Compiling jni-sys v0.3.1
   Compiling once_cell v1.21.4
   Compiling tempfile v3.27.0
   Compiling rand_core v0.6.4
   Compiling ppv-lite86 v0.2.21
   Compiling jni v0.20.0
   Compiling wait-timeout v0.2.1
   Compiling iana-time-zone v0.1.65
   Compiling bit-vec v0.8.0
   Compiling log v0.4.33
   Compiling fnv v1.0.7
   Compiling memchr v2.8.2
   Compiling jni-sys-macros v0.4.1
   Compiling thiserror-impl v1.0.69
   Compiling quick-error v1.2.3
   Compiling bytes v1.12.0
   Compiling jni-sys v0.4.1
   Compiling combine v4.6.7
   Compiling rusty-fork v0.3.1
   Compiling bit-set v0.8.0
   Compiling chrono v0.4.45
   Compiling rand_chacha v0.3.1
   Compiling rand_chacha v0.9.0
   Compiling rand_xorshift v0.4.0
   Compiling rand v0.9.4
   Compiling regex-syntax v0.8.11
   Compiling termcolor v1.1.3
   Compiling cesu8 v1.1.0
   Compiling unarray v0.1.4
   Compiling simplelog v0.11.2
   Compiling rand v0.8.6
   Compiling framework v0.1.0 (/home/user/cache-evolve/framework-rs)
   Compiling proptest v1.11.0
   Compiling associative_cache v0.1.0 (/tmp/variant-workspace-f8k2663p)
error[E0308]: mismatched types
   --> src/associative_cache/mod.rs:113:31
    |
113 |                 self.cached_result = Some(item);
    |                                      ---- ^^^^ expected `Item<K, V>`, found `&Item<K, V>`
    |                                      |
    |                                      arguments to this enum variant are incorrect
    |
    = note: expected struct `item::Item<_, _>`
            found reference `&item::Item<_, _>`
help: the type constructed contains `&item::Item<K, V>` due to the type of the argument passed
   --> src/associative_cache/mod.rs:113:26
    |
113 |                 self.cached_result = Some(item);
    |                                      ^^^^^----^
    |                                           |
    |                                           this argument influences the type of `Some`
note: tuple variant defined here
   --> /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/option.rs:608:4
help: consider using clone here
    |
113 |                 self.cached_result = Some(item.clone());
    |                                               ++++++++

For more information about this error, try `rustc --explain E0308`.
error: could not compile `associative_cache` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `associative_cache` (lib test) due to 1 previous error

```
---
