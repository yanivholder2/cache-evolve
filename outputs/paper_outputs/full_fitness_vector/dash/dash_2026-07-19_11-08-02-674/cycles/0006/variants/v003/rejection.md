
## 2026-07-20T16:59:38Z — variant=cycle/0006/v003 hypothesis=H03 attempt=1

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
   Compiling zerocopy v0.8.54
   Compiling autocfg v1.5.1
   Compiling windows-sys v0.61.2
   Compiling getrandom v0.4.3
   Compiling getrandom v0.2.17
   Compiling fastrand v2.5.0
   Compiling jni-sys v0.3.1
   Compiling num-traits v0.2.19
   Compiling thiserror v1.0.69
   Compiling rand_core v0.9.5
   Compiling once_cell v1.21.4
   Compiling rand_core v0.6.4
   Compiling bit-vec v0.8.0
   Compiling bytes v1.12.1
   Compiling winapi-util v0.1.11
   Compiling syn v2.0.119
   Compiling same-file v1.0.6
   Compiling tempfile v3.27.0
   Compiling walkdir v2.5.0
   Compiling memchr v2.8.3
   Compiling quick-error v1.2.3
   Compiling wait-timeout v0.2.1
   Compiling log v0.4.33
   Compiling fnv v1.0.7
   Compiling jni v0.20.0
   Compiling chrono v0.4.45
   Compiling rusty-fork v0.3.1
   Compiling combine v4.6.7
   Compiling termcolor v1.1.3
   Compiling bit-set v0.8.0
   Compiling rand v0.9.5
   Compiling rand_xorshift v0.4.0
   Compiling cesu8 v1.1.0
   Compiling bitflags v2.13.1
   Compiling regex-syntax v0.8.11
   Compiling unarray v0.1.4
   Compiling simplelog v0.11.2
   Compiling framework v0.1.0 (C:\Users\yaniv\Projects\cache-evolve\framework-rs)
   Compiling jni-sys-macros v0.4.1
   Compiling thiserror-impl v1.0.69
   Compiling ppv-lite86 v0.2.21
   Compiling jni-sys v0.4.1
   Compiling rand_chacha v0.9.0
   Compiling rand_chacha v0.3.1
   Compiling proptest v1.11.0
   Compiling rand v0.8.7
   Compiling dash v0.1.0 (C:\Users\yaniv\AppData\Local\Temp\variant-workspace-fnudpu6g)
error[E0277]: the trait bound `K: Hash` is not satisfied
  --> src\dash\segments\dash_segment.rs:79:46
   |
79 | fn clock_evict_prepare<K, V>(items: &mut Vec<Item<K, V>>)
   |                                              ^^^^^^^^^^ the trait `Hash` is not implemented for `K`
   |
note: required by a bound in `item::Item`
  --> src\shared\item.rs:22:5
   |
20 | pub struct Item<K, V>
   |            ---- required by a bound in this struct
21 | where
22 |     K: Hash + Eq + Clone,
   |        ^^^^ required by this bound in `Item`
help: consider further restricting type parameter `K` with trait `Hash`
   |
81 |     K: Clone + std::hash::Hash,
   |              +++++++++++++++++

error[E0277]: the trait bound `K: Eq` is not satisfied
  --> src\dash\segments\dash_segment.rs:79:46
   |
79 | fn clock_evict_prepare<K, V>(items: &mut Vec<Item<K, V>>)
   |                                              ^^^^^^^^^^ the trait `Eq` is not implemented for `K`
   |
note: required by a bound in `item::Item`
  --> src\shared\item.rs:22:12
   |
20 | pub struct Item<K, V>
   |            ---- required by a bound in this struct
21 | where
22 |     K: Hash + Eq + Clone,
   |               ^^ required by this bound in `Item`
help: consider further restricting type parameter `K` with trait `Eq`
   |
81 |     K: Clone + std::cmp::Eq,
   |              ++++++++++++++

error[E0277]: the trait bound `V: Eq` is not satisfied
  --> src\dash\segments\dash_segment.rs:79:46
   |
79 | fn clock_evict_prepare<K, V>(items: &mut Vec<Item<K, V>>)
   |                                              ^^^^^^^^^^ the trait `Eq` is not implemented for `V`
   |
note: required by a bound in `item::Item`
  --> src\shared\item.rs:23:5
   |
20 | pub struct Item<K, V>
   |            ---- required by a bound in this struct
...
23 |     V: Eq + Clone,
   |        ^^ required by this bound in `Item`
help: consider further restricting type parameter `V` with trait `Eq`
   |
82 |     V: Clone + std::cmp::Eq,
   |              ++++++++++++++

error[E0277]: the trait bound `K: Hash` is not satisfied
  --> src\dash\segments\dash_segment.rs:84:10
   |
84 |     let n = items.len();
   |             ^^^^^^^^^^^ the trait `Hash` is not implemented for `K`
   |
note: required by a bound in `item::Item`
  --> src\shared\item.rs:22:5
   |
20 | pub struct Item<K, V>
   |            ---- required by a bound in this struct
21 | where
22 |     K: Hash + Eq + Clone,
   |        ^^^^ required by this bound in `Item`
help: consider further restricting type parameter `K` with trait `Hash`
   |
81 |     K: Clone + std::hash::Hash,
   |              +++++++++++++++++

error[E0277]: the trait bound `K: Eq` is not satisfied
  --> src\dash\segments\dash_segment.rs:84:10
   |
84 |     let n = items.len();
   |             ^^^^^^^^^^^ the trait `Eq` is not implemented for `K`
   |
note: required by a bound in `item::Item`
  --> src\shared\item.rs:22:12
   |
20 | pub struct Item<K, V>
   |            ---- required by a bound in this struct
21 | where
22 |     K: Hash + Eq + Clone,
   |               ^^ required by this bound in `Item`
help: consider further restricting type parameter `K` with trait `Eq`
   |
81 |     K: Clone + std::cmp::Eq,
   |              ++++++++++++++

error[E0277]: the trait bound `V: Eq` is not satisfied
  --> src\dash\segments\dash_segment.rs:84:10
   |
84 |     let n = items.len();
   |             ^^^^^^^^^^^ the trait `Eq` is not implemented for `V`
   |
note: required by a bound in `item::Item`
  --> src\shared\item.rs:23:5
   |
20 | pub struct Item<K, V>
   |            ---- required by a bound in this struct
...
23 |     V: Eq + Clone,
   |        ^^ required by this bound in `Item`
help: consider further restricting type parameter `V` with trait `Eq`
   |
82 |     V: Clone + std::cmp::Eq,
   |              ++++++++++++++

error[E0277]: the trait bound `K: Hash` is not satisfied
  --> src\dash\segments\dash_segment.rs:86:6
   |
86 |         if items[0].lfu_counter == 0 {
   |            ^^^^^^^^ the trait `Hash` is not implemented for `K`
   |
note: required by a bound in `item::Item`
  --> src\shared\item.rs:22:5
   |
20 | pub struct Item<K, V>
   |            ---- required by a bound in this struct
21 | where
22 |     K: Hash + Eq + Clone,
   |        ^^^^ required by this bound in `Item`
help: consider further restricting type parameter `K` with trait `Hash`
   |
81 |     K: Clone + std::hash::Hash,
   |              +++++++++++++++++

error[E0277]: the trait bound `K: Eq` is not satisfied
  --> src\dash\segments\dash_segment.rs:86:6
   |
86 |         if items[0].lfu_counter == 0 {
   |            ^^^^^^^^ the trait `Eq` is not implemented for `K`
   |
note: required by a bound in `item::Item`
  --> src\shared\item.rs:22:12
   |
20 | pub struct Item<K, V>
   |            ---- required by a bound in this struct
21 | where
22 |     K: Hash + Eq + Clone,
   |               ^^ required by this bound in `Item`
help: consider further restricting type parameter `K` with trait `Eq`
   |
81 |     K: Clone + std::cmp::Eq,
   |              ++++++++++++++

error[E0277]: the trait bound `V: Eq` is not satisfied
  --> src\dash\segments\dash_segment.rs:86:6
   |
86 |         if items[0].lfu_counter == 0 {
   |            ^^^^^^^^ the trait `Eq` is not implemented for `V`
   |
note: required by a bound in `item::Item`
  --> src\shared\item.rs:23:5
   |
20 | pub struct Item<K, V>
   |            ---- required by a bound in this struct
...
23 |     V: Eq + Clone,
   |        ^^ required by this bound in `Item`
help: consider further restricting type parameter `V` with trait `Eq`
   |
82 |     V: Clone + std::cmp::Eq,
   |              ++++++++++++++

error[E0277]: the trait bound `K: Hash` is not satisfied
  --> src\dash\segments\dash_segment.rs:89:3
   |
89 |         items[0].lfu_counter -= 1;
   |         ^^^^^^^^ the trait `Hash` is not implemented for `K`
   |
note: required by a bound in `item::Item`
  --> src\shared\item.rs:22:5
   |
20 | pub struct Item<K, V>
   |            ---- required by a bound in this struct
21 | where
22 |     K: Hash + Eq + Clone,
   |        ^^^^ required by this bound in `Item`
help: consider further restricting type parameter `K` with trait `Hash`
   |
81 |     K: Clone + std::hash::Hash,
   |              +++++++++++++++++

error[E0277]: the trait bound `K: Eq` is not satisfied
  --> src\dash\segments\dash_segment.rs:89:3
   |
89 |         items[0].lfu_counter -= 1;
   |         ^^^^^^^^ the trait `Eq` is not implemented for `K`
   |
note: required by a bound in `item::Item`
  --> src\shared\item.rs:22:12
   |
20 | pub struct Item<K, V>
   |            ---- required by a bound in this struct
21 | where
22 |     K: Hash + Eq + Clone,
   |               ^^ required by this bound in `Item`
help: consider further restricting type parameter `K` with trait `Eq`
   |
81 |     K: Clone + std::cmp::Eq,
   |              ++++++++++++++

error[E0277]: the trait bound `V: Eq` is not satisfied
  --> src\dash\segments\dash_segment.rs:89:3
   |
89 |         items[0].lfu_counter -= 1;
   |         ^^^^^^^^ the trait `Eq` is not implemented for `V`
   |
note: required by a bound in `item::Item`
  --> src\shared\item.rs:23:5
   |
20 | pub struct Item<K, V>
   |            ---- required by a bound in this struct
...
23 |     V: Eq + Clone,
   |        ^^ required by this bound in `Item`
help: consider further restricting type parameter `V` with trait `Eq`
   |
82 |     V: Clone + std::cmp::Eq,
   |              ++++++++++++++

For more information about this error, try `rustc --explain E0277`.
error: could not compile `dash` (lib) due to 12 previous errors
warning: build failed, waiting for other jobs to finish...
error: could not compile `dash` (lib test) due to 12 previous errors

```
---
