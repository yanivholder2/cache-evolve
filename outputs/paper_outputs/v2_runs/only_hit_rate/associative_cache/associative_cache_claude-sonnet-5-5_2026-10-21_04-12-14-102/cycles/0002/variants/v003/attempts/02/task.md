# Task: variant-implementer

Variant: v003
Parent variant: baseline (/home/user/cache-evolve/inputs/subjects/associative_cache/baseline)
Hypothesis: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0002/hypotheses/H03.md
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0002/variants/v003/attempts/02/overlay (overlay-by-path)

## Paths

- Parent overlay/:       /home/user/cache-evolve/inputs/subjects/associative_cache/baseline/overlay
- Parent fitness-vector: /home/user/cache-evolve/inputs/subjects/associative_cache/baseline/eval/fitness-vector.json
- Parent insight.md:     /home/user/cache-evolve/inputs/subjects/associative_cache/baseline/insight.md
- Subject src/:          /home/user/cache-evolve/inputs/subjects/associative_cache/src
- Subject manifest:      /home/user/cache-evolve/inputs/subjects/associative_cache/manifest.toml

## Authoring scope

full-autonomy

Gate-integrity files are pinned to the pristine crate and may not be
authored in any scope — overlay copies of them are ignored and rejected:
src/associative_cache/associative_cache_jni.rs, src/factory.rs, Cargo.toml, src/lib.rs, tests/, src/test_utils/, src/bin/

## Failing gate
The previous attempt's overlay was rejected. Diagnose and fix it:
- Previous overlay (the parent you are fixing): /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0002/variants/v003/attempts/01/overlay
- Failing gate's test scaffold: /home/user/cache-evolve/inputs/subjects/associative_cache/tests/invariant_gate.rs

Latest rejection entry:

## 2026-10-21T04:16:46Z — variant=cycle/0002/v003 hypothesis=H03 attempt=1

**Gate**: invariant gate
**Failing invariant**: P2 slot_legality
**Error**:
```

running 6 tests
test capacity_conservation_detects_inflated_geometry ... ok
test capacity_bound_holds_under_overfill ... ok
test capacity_conservation_holds_for_baseline_geometry ... ok
test slot_legality_holds_under_workload ... ok
test baseline_passes_all_invariants ... FAILED
test associative_invariants_runs_baseline_suite ... FAILED

failures:

---- baseline_passes_all_invariants stdout ----
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known

thread 'baseline_passes_all_invariants' (12224) panicked at tests/invariant_gate.rs:92:46:
called `Result::unwrap()` on an `Err` value: "invariant gate violated: P2 slot_legality: Test failed: slot_legality violated: key=172 sits in bucket 3 but hashes to 0 at src/test_utils/invariants.rs:75.\nminimal failing input: [\n    22,\n    172,\n]"
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

thread 'associative_invariants_runs_baseline_suite' (12223) panicked at tests/invariant_gate.rs:98:55:
called `Result::unwrap()` on an `Err` value: "invariant gate violated: P2 slot_legality: Test failed: slot_legality violated: key=172 sits in bucket 3 but hashes to 0 at src/test_utils/invariants.rs:75.\nminimal failing input: [\n    56,\n    172,\n]"
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

test result: FAILED. 4 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s

    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/invariant_gate.rs (/tmp/variant-workspace-egw3rtid/target/debug/deps/invariant_gate-9285ab2ce564358e)
error: test failed, to rerun pass `--test invariant_gate`

```
---

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied to a copy of the subject crate at build time (gate-integrity
files excluded).
