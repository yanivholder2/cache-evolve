
## 2026-10-21T09:34:16Z — variant=cycle/0009/v001 hypothesis=H01 attempt=1

**Gate**: invariant gate
**Failing invariant**: P2 slot_legality
**Error**:
```

running 6 tests
test capacity_conservation_detects_inflated_geometry ... ok
test capacity_conservation_holds_for_baseline_geometry ... ok
test capacity_bound_holds_under_overfill ... ok
test slot_legality_holds_under_workload ... FAILED
test baseline_passes_all_invariants ... FAILED
test associative_invariants_runs_baseline_suite ... FAILED

failures:

---- slot_legality_holds_under_workload stdout ----

thread 'slot_legality_holds_under_workload' (1265) panicked at tests/invariant_gate.rs:52:13:
assertion `left == right` failed: key 12 sits in bucket 3 but hashes elsewhere
  left: 0
 right: 3
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/panicking.rs:80:14
   2: core::panicking::assert_failed_inner
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/panicking.rs:434:23
   3: core::panicking::assert_failed::<usize, usize>
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/panicking.rs:394:5
   4: invariant_gate::slot_legality_holds_under_workload
             at ./tests/invariant_gate.rs:52:4
   5: invariant_gate::slot_legality_holds_under_workload::{closure#0}
             at ./tests/invariant_gate.rs:41:40
   6: <invariant_gate::slot_legality_holds_under_workload::{closure#0} as core::ops::function::FnOnce<()>>::call_once
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ops/function.rs:250:5
   7: <fn() -> core::result::Result<(), alloc::string::String> as core::ops::function::FnOnce<()>>::call_once
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.

---- baseline_passes_all_invariants stdout ----
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known

thread 'baseline_passes_all_invariants' (1261) panicked at tests/invariant_gate.rs:92:46:
called `Result::unwrap()` on an `Err` value: "invariant gate violated: P2 slot_legality: Test failed: slot_legality violated: key=85 sits in bucket 2 but hashes to 1 at src/test_utils/invariants.rs:75.\nminimal failing input: [\n    92,\n    74,\n    118,\n    85,\n]"
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/panicking.rs:80:14
   2: core::result::unwrap_failed
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/result.rs:1867:5
   3: <core::result::Result<(), alloc::string::String>>::unwrap
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/result.rs:1233:23
   4: invariant_gate::baseline_passes_all_invariants
             at ./tests/invariant_gate.rs:92:43
   5: invariant_gate::baseline_passes_all_invariants::{closure#0}
             at ./tests/invariant_gate.rs:91:36
   6: <invariant_gate::baseline_passes_all_invariants::{closure#0} as core::ops::function::FnOnce<()>>::call_once
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ops/function.rs:250:5
   7: <fn() -> core::result::Result<(), alloc::string::String> as core::ops::function::FnOnce<()>>::call_once
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.

---- associative_invariants_runs_baseline_suite stdout ----
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known

thread 'associative_invariants_runs_baseline_suite' (1260) panicked at tests/invariant_gate.rs:98:55:
called `Result::unwrap()` on an `Err` value: "invariant gate violated: P2 slot_legality: Test failed: slot_legality violated: key=18 sits in bucket 2 but hashes to 3 at src/test_utils/invariants.rs:75.\nminimal failing input: [\n    41,\n    63,\n    174,\n    18,\n]"
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/panicking.rs:80:14
   2: core::result::unwrap_failed
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/result.rs:1867:5
   3: <core::result::Result<(), alloc::string::String>>::unwrap
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/result.rs:1233:23
   4: invariant_gate::associative_invariants_runs_baseline_suite
             at ./tests/invariant_gate.rs:98:52
   5: invariant_gate::associative_invariants_runs_baseline_suite::{closure#0}
             at ./tests/invariant_gate.rs:96:48
   6: <invariant_gate::associative_invariants_runs_baseline_suite::{closure#0} as core::ops::function::FnOnce<()>>::call_once
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ops/function.rs:250:5
   7: <fn() -> core::result::Result<(), alloc::string::String> as core::ops::function::FnOnce<()>>::call_once
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.


failures:
    associative_invariants_runs_baseline_suite
    baseline_passes_all_invariants
    slot_legality_holds_under_workload

test result: FAILED. 3 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s

    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/invariant_gate.rs (/tmp/variant-workspace-7n88qaj0/target/debug/deps/invariant_gate-9285ab2ce564358e)
error: test failed, to rerun pass `--test invariant_gate`

```
---

## 2026-10-21T09:34:46Z — variant=cycle/0009/v001 hypothesis=H01 attempt=2

**Stage**: implement

**Problem**: adapter produced no overlay files under /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0009/variants/v001/attempts/02/overlay

Write at least one changed subject file under overlay/; an empty overlay is indistinguishable from the baseline.

## 2026-10-21T09:36:50Z — variant=cycle/0009/v001 hypothesis=H01 attempt=3

**Gate**: invariant gate
**Failing invariant**: P2 slot_legality
**Error**:
```

running 6 tests
test capacity_conservation_detects_inflated_geometry ... ok
test capacity_conservation_holds_for_baseline_geometry ... ok
test capacity_bound_holds_under_overfill ... ok
test slot_legality_holds_under_workload ... FAILED
test baseline_passes_all_invariants ... FAILED
test associative_invariants_runs_baseline_suite ... FAILED

failures:

---- slot_legality_holds_under_workload stdout ----

thread 'slot_legality_holds_under_workload' (3198) panicked at tests/invariant_gate.rs:52:13:
assertion `left == right` failed: key 12 sits in bucket 3 but hashes elsewhere
  left: 0
 right: 3
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/panicking.rs:80:14
   2: core::panicking::assert_failed_inner
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/panicking.rs:434:23
   3: core::panicking::assert_failed::<usize, usize>
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/panicking.rs:394:5
   4: invariant_gate::slot_legality_holds_under_workload
             at ./tests/invariant_gate.rs:52:4
   5: invariant_gate::slot_legality_holds_under_workload::{closure#0}
             at ./tests/invariant_gate.rs:41:40
   6: <invariant_gate::slot_legality_holds_under_workload::{closure#0} as core::ops::function::FnOnce<()>>::call_once
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ops/function.rs:250:5
   7: <fn() -> core::result::Result<(), alloc::string::String> as core::ops::function::FnOnce<()>>::call_once
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.

---- baseline_passes_all_invariants stdout ----
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known

thread 'baseline_passes_all_invariants' (3194) panicked at tests/invariant_gate.rs:92:46:
called `Result::unwrap()` on an `Err` value: "invariant gate violated: P2 slot_legality: Test failed: slot_legality violated: key=87 sits in bucket 2 but hashes to 1 at src/test_utils/invariants.rs:75.\nminimal failing input: [\n    0,\n    0,\n    36,\n    85,\n    87,\n]"
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/panicking.rs:80:14
   2: core::result::unwrap_failed
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/result.rs:1867:5
   3: <core::result::Result<(), alloc::string::String>>::unwrap
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/result.rs:1233:23
   4: invariant_gate::baseline_passes_all_invariants
             at ./tests/invariant_gate.rs:92:43
   5: invariant_gate::baseline_passes_all_invariants::{closure#0}
             at ./tests/invariant_gate.rs:91:36
   6: <invariant_gate::baseline_passes_all_invariants::{closure#0} as core::ops::function::FnOnce<()>>::call_once
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ops/function.rs:250:5
   7: <fn() -> core::result::Result<(), alloc::string::String> as core::ops::function::FnOnce<()>>::call_once
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.

---- associative_invariants_runs_baseline_suite stdout ----
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known

thread 'associative_invariants_runs_baseline_suite' (3193) panicked at tests/invariant_gate.rs:98:55:
called `Result::unwrap()` on an `Err` value: "invariant gate violated: P2 slot_legality: Test failed: slot_legality violated: key=25 sits in bucket 0 but hashes to 1 at src/test_utils/invariants.rs:75.\nminimal failing input: [\n    92,\n    36,\n    74,\n    25,\n]"
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/panicking.rs:80:14
   2: core::result::unwrap_failed
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/result.rs:1867:5
   3: <core::result::Result<(), alloc::string::String>>::unwrap
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/result.rs:1233:23
   4: invariant_gate::associative_invariants_runs_baseline_suite
             at ./tests/invariant_gate.rs:98:52
   5: invariant_gate::associative_invariants_runs_baseline_suite::{closure#0}
             at ./tests/invariant_gate.rs:96:48
   6: <invariant_gate::associative_invariants_runs_baseline_suite::{closure#0} as core::ops::function::FnOnce<()>>::call_once
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ops/function.rs:250:5
   7: <fn() -> core::result::Result<(), alloc::string::String> as core::ops::function::FnOnce<()>>::call_once
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.


failures:
    associative_invariants_runs_baseline_suite
    baseline_passes_all_invariants
    slot_legality_holds_under_workload

test result: FAILED. 3 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/invariant_gate.rs (/tmp/variant-workspace-_ht1e6aa/target/debug/deps/invariant_gate-9285ab2ce564358e)
error: test failed, to rerun pass `--test invariant_gate`

```
---
