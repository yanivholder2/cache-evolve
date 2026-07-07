
## 2026-06-10T20:23:29Z — variant=cycle/0001/v002 hypothesis=H02 attempt=1

**Gate**: invariant gate
**Failing invariant**: P2 slot_legality
**Error**:
```

running 6 tests
test capacity_conservation_detects_inflated_geometry ... ok
test capacity_conservation_holds_for_baseline_geometry ... ok
test capacity_bound_holds_under_overfill ... ok
test slot_legality_holds_under_workload ... FAILED
test associative_invariants_runs_baseline_suite ... FAILED
test baseline_passes_all_invariants ... FAILED

failures:

---- slot_legality_holds_under_workload stdout ----

thread 'slot_legality_holds_under_workload' (9313) panicked at tests/invariant_gate.rs:52:13:
assertion `left == right` failed: key 483 sits in bucket 1 but hashes elsewhere
  left: 2
 right: 1
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:80:14
   2: core::panicking::assert_failed_inner
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:434:23
   3: core::panicking::assert_failed
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:394:5
   4: invariant_gate::slot_legality_holds_under_workload
             at ./tests/invariant_gate.rs:52:4
   5: invariant_gate::slot_legality_holds_under_workload::{{closure}}
             at ./tests/invariant_gate.rs:41:40
   6: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
   7: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.

---- associative_invariants_runs_baseline_suite stdout ----
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known

thread 'associative_invariants_runs_baseline_suite' (9308) panicked at tests/invariant_gate.rs:98:55:
called `Result::unwrap()` on an `Err` value: "invariant gate violated: P2 slot_legality: Test failed: slot_legality violated: key=29 sits in bucket 3 but hashes to 0 at src/test_utils/invariants.rs:75.\nminimal failing input: [\n    29,\n    172,\n    58,\n    103,\n]"
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:80:14
   2: core::result::unwrap_failed
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/result.rs:1867:5
   3: core::result::Result<T,E>::unwrap
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/result.rs:1233:23
   4: invariant_gate::associative_invariants_runs_baseline_suite
             at ./tests/invariant_gate.rs:98:52
   5: invariant_gate::associative_invariants_runs_baseline_suite::{{closure}}
             at ./tests/invariant_gate.rs:96:48
   6: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
   7: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.

---- baseline_passes_all_invariants stdout ----
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known

thread 'baseline_passes_all_invariants' (9309) panicked at tests/invariant_gate.rs:92:46:
called `Result::unwrap()` on an `Err` value: "invariant gate violated: P2 slot_legality: Test failed: slot_legality violated: key=165 sits in bucket 2 but hashes to 3 at src/test_utils/invariants.rs:75.\nminimal failing input: [\n    165,\n    60,\n    10,\n    63,\n]"
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:80:14
   2: core::result::unwrap_failed
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/result.rs:1867:5
   3: core::result::Result<T,E>::unwrap
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/result.rs:1233:23
   4: invariant_gate::baseline_passes_all_invariants
             at ./tests/invariant_gate.rs:92:43
   5: invariant_gate::baseline_passes_all_invariants::{{closure}}
             at ./tests/invariant_gate.rs:91:36
   6: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
   7: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.


failures:
    associative_invariants_runs_baseline_suite
    baseline_passes_all_invariants
    slot_legality_holds_under_workload

test result: FAILED. 3 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

warning: unused variable: `primary_bucket_id`
  --> src/associative_cache/mod.rs:90:78
   |
90 |     fn try_place_in_neighbor(&mut self, victim: Item<K, V>, neighbor_id: usize, primary_bucket_id: usize) -> bool {
   |                                                                                 ^^^^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_primary_bucket_id`
   |
   = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

warning: method `get_mut_bucket` is never used
   --> src/associative_cache/mod.rs:168:5
    |
 24 | / impl<K, V> AssociativeCache<K, V>
 25 | | where
 26 | |     K: Hash + Eq + Copy + Debug,
 27 | |     V: Eq + Copy + Debug,
    | |_________________________- method in this implementation
...
168 |       fn get_mut_bucket(&mut self, key: &K) -> &mut AssociativeCacheBucket<K, V> {
    |          ^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `associative_cache` (lib) generated 2 warnings (run `cargo fix --lib -p associative_cache` to apply 1 suggestion)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/invariant_gate.rs (/tmp/variant-workspace-wk_oc2b1/target/debug/deps/invariant_gate-414e19e4143ae900)
error: test failed, to rerun pass `--test invariant_gate`

```
---
