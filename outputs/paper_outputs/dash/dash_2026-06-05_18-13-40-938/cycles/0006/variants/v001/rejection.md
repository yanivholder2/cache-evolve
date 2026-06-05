
## 2026-06-05T19:02:24Z — variant=cycle/0006/v001 hypothesis=H01 attempt=1

**Gate**: contract gate
**Error**:
```

running 1 test
test contract_suite ... FAILED

failures:

---- contract_suite stdout ----

thread 'contract_suite' (17893) panicked at src/factory.rs:48:10:
geometry violated capacity conservation; the invariant gate should have rejected this variant: "budget 12 but geometry yields 64"
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:80:14
   2: core::result::unwrap_failed
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/result.rs:1867:5
   3: core::result::Result<T,E>::expect
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/result.rs:1185:23
   4: dash::factory::make_cache
             at ./src/factory.rs:48:4
   5: contract_gate::contract_suite::{{closure}}
             at ./tests/contract_gate.rs:23:24
   6: framework::test_suite::run_contract_suite
             at /home/user/cache-evolve/framework-rs/src/test_suite.rs:22:15
   7: contract_gate::contract_suite
             at ./tests/contract_gate.rs:23:2
   8: contract_gate::contract_suite::{{closure}}
             at ./tests/contract_gate.rs:22:20
   9: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
  10: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.


failures:
    contract_suite

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

warning: variable does not need to be mutable
   --> src/dash/segments/dash_segment.rs:144:15
    |
144 |         if let Some(mut cascade) = core.put_at(Slot::Target, InsertPosition::Back, item) {
    |                     ----^^^^^^^
    |                     |
    |                     help: remove this `mut`
    |
    = note: `#[warn(unused_mut)]` (part of `#[warn(unused)]`) on by default

warning: unused variable: `insert_pos`
   --> src/dash/segments/dash_segment.rs:124:2
    |
124 |     insert_pos: InsertPosition,
    |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_insert_pos`
    |
    = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

warning: `dash` (lib) generated 2 warnings (run `cargo fix --lib -p dash` to apply 2 suggestions)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/contract_gate.rs (/tmp/variant-workspace-oqzi8d7v/target/debug/deps/contract_gate-91e7803692e93999)
error: test failed, to rerun pass `--test contract_gate`

```
---

## 2026-06-05T19:03:42Z — variant=cycle/0006/v001 hypothesis=H01 attempt=2

**Gate**: invariant gate (harness error)
**Failing invariant**: <harness error — see Error block>
**Error**:
```

running 9 tests
test capacity_bound_negative_fixture ... ok
test capacity_conservation_detects_inflated_geometry ... ok
test capacity_conservation_holds_for_baseline_geometry ... ok
test single_eviction_per_put_negative_fixture ... ok
test no_phantom_items_negative_fixture ... ok
test slot_legality_negative_fixture ... ok
test stash_terminal_negative_fixture ... ok
test harness_tests::dash_segment_passes_all_invariants ... FAILED
test harness_tests::dash_invariants_runs_baseline_suite ... FAILED

failures:

---- harness_tests::dash_segment_passes_all_invariants stdout ----
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known

thread 'harness_tests::dash_segment_passes_all_invariants' (19416) panicked at tests/invariant_gate.rs:165:54:
called `Result::unwrap()` on an `Err` value: "P6 capacity_conservation: Test failed: P6 capacity_conservation violated: geometry produced an invalid topology: num_of_normal_buckets=0 (must be >= 2 for probing) at src/test_utils/invariants.rs:184.\nminimal failing input: DashSettings {\n    num_of_segments: 1,\n    num_of_normal_buckets: 2,\n    num_of_stash_buckets: 1,\n    bucket_size: 1,\n    eviction_policy: ClassicLRU,\n    debug_mode: 0,\n}"
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:80:14
   2: core::result::unwrap_failed
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/result.rs:1867:5
   3: core::result::Result<T,E>::unwrap
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/result.rs:1233:23
   4: invariant_gate::harness_tests::dash_segment_passes_all_invariants
             at ./tests/invariant_gate.rs:165:48
   5: invariant_gate::harness_tests::dash_segment_passes_all_invariants::{{closure}}
             at ./tests/invariant_gate.rs:163:41
   6: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
   7: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.

---- harness_tests::dash_invariants_runs_baseline_suite stdout ----
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known

thread 'harness_tests::dash_invariants_runs_baseline_suite' (19415) panicked at tests/invariant_gate.rs:172:47:
called `Result::unwrap()` on an `Err` value: "P6 capacity_conservation: Test failed: P6 capacity_conservation violated: geometry produced an invalid topology: num_of_normal_buckets=0 (must be >= 2 for probing) at src/test_utils/invariants.rs:184.\nminimal failing input: DashSettings {\n    num_of_segments: 1,\n    num_of_normal_buckets: 2,\n    num_of_stash_buckets: 1,\n    bucket_size: 1,\n    eviction_policy: ClassicLRU,\n    debug_mode: 0,\n}"
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:80:14
   2: core::result::unwrap_failed
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/result.rs:1867:5
   3: core::result::Result<T,E>::unwrap
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/result.rs:1233:23
   4: invariant_gate::harness_tests::dash_invariants_runs_baseline_suite
             at ./tests/invariant_gate.rs:172:41
   5: invariant_gate::harness_tests::dash_invariants_runs_baseline_suite::{{closure}}
             at ./tests/invariant_gate.rs:169:42
   6: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
   7: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.


failures:
    harness_tests::dash_invariants_runs_baseline_suite
    harness_tests::dash_segment_passes_all_invariants

test result: FAILED. 7 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s

    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/invariant_gate.rs (/tmp/variant-workspace-ahwl6mm1/target/debug/deps/invariant_gate-7c505ed1f391d40d)
error: test failed, to rerun pass `--test invariant_gate`

```
---
