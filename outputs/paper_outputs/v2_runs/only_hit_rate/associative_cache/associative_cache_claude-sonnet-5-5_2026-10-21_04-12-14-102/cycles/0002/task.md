# Task: cycle-insight-writer

Cycle: 0002
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0002

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0002/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0002/variants/v002/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0002/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0002/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0002/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three variants tried: wider buckets (geometry, v001), cheaper one-step hit promotion (v002), and 2Q-style half-promotion for scan resistance (v003). Only the geometry change (v001) won, with a small hit-rate gain. Both policy-tweak variants slightly lowered hit rate versus plain move-to-back LRU.

## Patterns

- Only hit-rate was reliably measured. Per-op-time and metadata-size were missing from every variant's fitness vector, so the speed and size claims in all three hypotheses went untested.
- Both variants that weakened promotion in short buckets (v002, v003) lost hit rate. Full move-to-back recency seems hard to beat at small associativity on this LIRS trace.
- The geometry gain was marginal, so hash imbalance across sets is a minor source of misses. Most misses look like capacity or policy misses.
- Winning by a small hit-rate margin is unproven, because the likely timing cost of 64-way scans and shifts was never checked.

## Open questions

- Why was per-op-time not reported, and can the evaluation be fixed so speed-targeting hypotheses can be judged?
- Does the wider-bucket gain survive once get/remove/push cost is measured?
- Would scan resistance work with a separate probationary region instead of a shallower promotion within one short bucket?

## Continuation

Build on v001's geometry, but first make sure per-op-time and metadata-size are measured so trade-offs can be judged. Drop promotion-weakening ideas (transposition, midpoint) in plain LRU buckets. Next, try policies that change eviction or admission (e.g. a separate probationary segment, frequency-aware victim choice) rather than hit-path promotion.


## Rejected variants

- v003 — gate:invariant gate (3/3 attempts)
  ## 2026-10-21T04:18:04Z — variant=cycle/0002/v003 hypothesis=H03 attempt=3
  
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
  
  thread 'slot_legality_holds_under_workload' (17261) panicked at tests/invariant_gate.rs:52:13:
  assertion `left == right` failed: key 495 sits in bucket 0 but hashes elsewhere
    left: 3
   right: 0
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
  
  thread 'baseline_passes_all_invariants' (17257) panicked at tests/invariant_gate.rs:92:46:
  called `Result::unwrap()` on an `Err` value: "invariant gate violated: P2 slot_legality: Test failed: slot_legality violated: key=139 sits in bucket 0 but hashes to 2 at src/test_utils/invariants.rs:75.\nminimal failing input: [\n    138,\n    139,\n]"
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
  
  thread 'associative_invariants_runs_baseline_suite' (17256) panicked at tests/invariant_gate.rs:98:55:
  called `Result::unwrap()` on an `Err` value: "invariant gate violated: P2 slot_legality: Test failed: slot_legality violated: key=152 sits in bucket 1 but hashes to 2 at src/test_utils/invariants.rs:75.\nminimal failing input: [\n    11,\n    152,\n]"
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
  
  test result: FAILED. 3 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
  
      Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
       Running tests/invariant_gate.rs (/tmp/variant-workspace-wup20zjp/target/debug/deps/invariant_gate-9285ab2ce564358e)
  error: test failed, to rerun pass `--test invariant_gate`
  
  ```
  ---

## Write

Exactly one file: `cycle-insight.md` in the output dir.
