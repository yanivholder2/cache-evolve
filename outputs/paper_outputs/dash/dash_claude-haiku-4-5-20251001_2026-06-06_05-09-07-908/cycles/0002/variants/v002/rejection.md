
## 2026-06-06T05:23:45Z — variant=cycle/0002/v002 hypothesis=H02 attempt=1

**Gate**: invariant gate (harness error)
**Failing invariant**: <harness error — see Error block>
**Error**:
```

running 9 tests
test capacity_bound_negative_fixture ... ok
test capacity_conservation_detects_inflated_geometry ... ok
test capacity_conservation_holds_for_baseline_geometry ... ok
test no_phantom_items_negative_fixture ... ok
test single_eviction_per_put_negative_fixture ... ok
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

thread 'harness_tests::dash_segment_passes_all_invariants' (21183) panicked at tests/invariant_gate.rs:165:54:
called `Result::unwrap()` on an `Err` value: "P6 capacity_conservation: Test failed: P6 capacity_conservation violated: geometry produced an invalid topology: num_of_normal_buckets=1 (must be >= 2 for probing) at src/test_utils/invariants.rs:184.\nminimal failing input: DashSettings {\n    num_of_segments: 1,\n    num_of_normal_buckets: 2,\n    num_of_stash_buckets: 1,\n    bucket_size: 1,\n    eviction_policy: ClassicLRU,\n    debug_mode: 0,\n}"
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

thread 'harness_tests::dash_invariants_runs_baseline_suite' (21182) panicked at tests/invariant_gate.rs:172:47:
called `Result::unwrap()` on an `Err` value: "P6 capacity_conservation: Test failed: P6 capacity_conservation violated: geometry produced an invalid topology: num_of_normal_buckets=1 (must be >= 2 for probing) at src/test_utils/invariants.rs:184.\nminimal failing input: DashSettings {\n    num_of_segments: 1,\n    num_of_normal_buckets: 2,\n    num_of_stash_buckets: 1,\n    bucket_size: 1,\n    eviction_policy: ClassicLRU,\n    debug_mode: 0,\n}"
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

test result: FAILED. 7 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s

    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/invariant_gate.rs (/tmp/variant-workspace-g1m4tkfk/target/debug/deps/invariant_gate-4d86cea63c479d8c)
error: test failed, to rerun pass `--test invariant_gate`

```
---
