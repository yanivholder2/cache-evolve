
## 2026-07-04T20:58:07Z — variant=cycle/0010/v003 hypothesis=H03 attempt=1

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
   Compiling once_cell v1.21.4
   Compiling fastrand v2.4.1
   Compiling jni-sys v0.3.1
   Compiling thiserror v1.0.69
   Compiling tempfile v3.27.0
   Compiling ppv-lite86 v0.2.21
   Compiling rand_core v0.6.4
   Compiling jni v0.20.0
   Compiling wait-timeout v0.2.1
   Compiling log v0.4.33
   Compiling iana-time-zone v0.1.65
   Compiling quick-error v1.2.3
   Compiling bytes v1.12.0
   Compiling bit-vec v0.8.0
   Compiling fnv v1.0.7
   Compiling jni-sys-macros v0.4.1
   Compiling thiserror-impl v1.0.69
   Compiling memchr v2.8.2
   Compiling jni-sys v0.4.1
   Compiling combine v4.6.7
   Compiling bit-set v0.8.0
   Compiling rusty-fork v0.3.1
   Compiling chrono v0.4.45
   Compiling rand_chacha v0.9.0
   Compiling rand_chacha v0.3.1
   Compiling rand_xorshift v0.4.0
   Compiling rand v0.9.4
   Compiling cesu8 v1.1.0
   Compiling termcolor v1.1.3
   Compiling unarray v0.1.4
   Compiling regex-syntax v0.8.11
   Compiling simplelog v0.11.2
   Compiling rand v0.8.6
   Compiling framework v0.1.0 (/home/user/cache-evolve/framework-rs)
   Compiling proptest v1.11.0
   Compiling associative_cache v0.1.0 (/tmp/variant-workspace-8q1qnwmi)
error[E0502]: cannot borrow `self.hot_stash` as immutable because it is also borrowed as mutable
   --> src/associative_cache/mod.rs:128:7
    |
 93 |     pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
    |                                - let's call the lifetime of this reference `'1`
...
105 |         for item in &mut self.hot_stash {
    |                     ------------------- mutable borrow occurs here
...
110 |                 return Some(&item.value);
    |                        ----------------- returning this value requires that `self.hot_stash` is borrowed for `'1`
...
128 |             if self.hot_stash.len() < self.hot_stash_capacity {
    |                ^^^^^^^^^^^^^^ immutable borrow occurs here

error[E0499]: cannot borrow `self.hot_stash` as mutable more than once at a time
   --> src/associative_cache/mod.rs:135:5
    |
 93 |     pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
    |                                - let's call the lifetime of this reference `'1`
...
105 |         for item in &mut self.hot_stash {
    |                     ------------------- first mutable borrow occurs here
...
110 |                 return Some(&item.value);
    |                        ----------------- returning this value requires that `self.hot_stash` is borrowed for `'1`
...
135 |                 self.hot_stash.push(promoted_item);
    |                 ^^^^^^^^^^^^^^ second mutable borrow occurs here

error[E0502]: cannot borrow `self.hot_stash` as immutable because it is also borrowed as mutable
   --> src/associative_cache/mod.rs:136:18
    |
 93 |     pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
    |                                - let's call the lifetime of this reference `'1`
...
105 |         for item in &mut self.hot_stash {
    |                     ------------------- mutable borrow occurs here
...
110 |                 return Some(&item.value);
    |                        ----------------- returning this value requires that `self.hot_stash` is borrowed for `'1`
...
136 |                 return Some(&self.hot_stash.last().unwrap().value);
    |                              ^^^^^^^^^^^^^^ immutable borrow occurs here

error[E0502]: cannot borrow `self.hot_stash` as immutable because it is also borrowed as mutable
   --> src/associative_cache/mod.rs:137:14
    |
 93 |     pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
    |                                - let's call the lifetime of this reference `'1`
...
105 |         for item in &mut self.hot_stash {
    |                     ------------------- mutable borrow occurs here
...
110 |                 return Some(&item.value);
    |                        ----------------- returning this value requires that `self.hot_stash` is borrowed for `'1`
...
137 |             } else if self.hot_stash.len() > 0 {
    |                       ^^^^^^^^^^^^^^ immutable borrow occurs here

error[E0502]: cannot borrow `self.hot_stash` as immutable because it is also borrowed as mutable
   --> src/associative_cache/mod.rs:160:25
    |
 93 |     pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
    |                                - let's call the lifetime of this reference `'1`
...
105 |         for item in &mut self.hot_stash {
    |                     ------------------- mutable borrow occurs here
...
110 |                 return Some(&item.value);
    |                        ----------------- returning this value requires that `self.hot_stash` is borrowed for `'1`
...
160 |                     for (idx, item) in self.hot_stash.iter().enumerate() {
    |                                        ^^^^^^^^^^^^^^ immutable borrow occurs here

error[E0502]: cannot borrow `self.hot_stash` as immutable because it is also borrowed as mutable
   --> src/associative_cache/mod.rs:161:27
    |
 93 |     pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
    |                                - let's call the lifetime of this reference `'1`
...
105 |         for item in &mut self.hot_stash {
    |                     ------------------- mutable borrow occurs here
...
110 |                 return Some(&item.value);
    |                        ----------------- returning this value requires that `self.hot_stash` is borrowed for `'1`
...
161 |                         if item.timestamp < self.hot_stash[lru_index].timestamp {
    |                                             ^^^^^^^^^^^^^^ immutable borrow occurs here

error[E0499]: cannot borrow `self.hot_stash` as mutable more than once at a time
   --> src/associative_cache/mod.rs:166:29
    |
 93 |     pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
    |                                - let's call the lifetime of this reference `'1`
...
105 |         for item in &mut self.hot_stash {
    |                     ------------------- first mutable borrow occurs here
...
110 |                 return Some(&item.value);
    |                        ----------------- returning this value requires that `self.hot_stash` is borrowed for `'1`
...
166 |                     let evicted_from_hot = self.hot_stash.remove(lru_index);
    |                                            ^^^^^^^^^^^^^^ second mutable borrow occurs here

error[E0499]: cannot borrow `self.hot_stash` as mutable more than once at a time
   --> src/associative_cache/mod.rs:175:6
    |
 93 |     pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
    |                                - let's call the lifetime of this reference `'1`
...
105 |         for item in &mut self.hot_stash {
    |                     ------------------- first mutable borrow occurs here
...
110 |                 return Some(&item.value);
    |                        ----------------- returning this value requires that `self.hot_stash` is borrowed for `'1`
...
175 |                     self.hot_stash.push(promoted_item);
    |                     ^^^^^^^^^^^^^^ second mutable borrow occurs here

error[E0594]: cannot assign to `evicted_from_hot.cold_access_count`, as `evicted_from_hot` is not declared as mutable
   --> src/associative_cache/mod.rs:178:7
    |
178 |                         evicted_from_hot.cold_access_count = 0;
    |                         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ cannot assign
    |
help: consider changing this to be mutable
    |
166 |                     let mut evicted_from_hot = self.hot_stash.remove(lru_index);
    |                         +++

error[E0502]: cannot borrow `self.hot_stash` as immutable because it is also borrowed as mutable
   --> src/associative_cache/mod.rs:184:20
    |
 93 |     pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
    |                                - let's call the lifetime of this reference `'1`
...
105 |         for item in &mut self.hot_stash {
    |                     ------------------- mutable borrow occurs here
...
110 |                 return Some(&item.value);
    |                        ----------------- returning this value requires that `self.hot_stash` is borrowed for `'1`
...
184 |                         return Some(&self.hot_stash.last().unwrap().value);
    |                                      ^^^^^^^^^^^^^^ immutable borrow occurs here

Some errors have detailed explanations: E0499, E0502, E0594.
For more information about an error, try `rustc --explain E0499`.
error: could not compile `associative_cache` (lib) due to 10 previous errors
warning: build failed, waiting for other jobs to finish...
error: could not compile `associative_cache` (lib test) due to 10 previous errors

```
---
