# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0009

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants were narrow follow-ups to Cycle 0005's continuation. v001 shrank the GCLOCK ghost ring from 4 to 2. v002 retried the leniency duel with DIP-style fixed leader buckets and a global PSEL. v003 limited ghost recording on the 1/8 LRU line to keys that were never hit. All three hypotheses were refuted and no variant beat its parent. v002 is the listed winner only because it lost the least. It adds no new mechanism worth keeping, and its own insight argues for dropping the duel.

## Patterns

- Every tweak to the ghost signal lost a little, so the current settings are already locally optimal. On ring size, 2 lost slightly and 16 (Cycle 0005) lost more, so the peak is at or near 4. On ring admission, filtering out previously-hit keys removed useful ghost hits on the LRU line. Ring length, ring admission and newcomer grace have now each been probed from both sides on workload-a.
- Newcomer leniency is closed on workload-a. Even with fixed leaders that removed the self-referential bias from Cycle 0005, the lenient one-sweep grace did worse than strict hand-on-newcomer. The duel also costs hits just by keeping lenient leaders running. Capped ghost admission already gives returning keys all the protection they need.
- Lessons about ghosts do not carry over cleanly between lines. Cycle 0005's "narrow the ring" result came from CLOCK, where long-reuse keys diluted the ring. On LRU, a hit key that has aged to index 0 has a reuse distance just past the bucket, which is exactly the population the ghost reward is meant to rescue. The two lines' ghost rings select different populations, so the same filtering rule behaves differently on each.
- All losses this cycle were small, and some may be within noise on one trace at one size. Workload-a hit rate looks saturated for the current design family.
- After six cycles, per-op-time and metadata-size are still not recorded. That is why v001's real saving (half the ghost memory, half the miss-path scan) counted for nothing. With hit rate saturated, these unmeasured dimensions are now what decides between candidates.

## Open questions

- Does a 3-entry ring match 4? This only matters once metadata-size or miss-path cost is measured.
- Does strict filtering plus a capped ghost counter hold up on a second trace or cache size, especially one with longer reuse distances?
- On the LRU line, would changing ghost-hit insert depth or ring length help where changing ring admission did not?

## Continuation

Treat strict hand-on-newcomer plus a 4-entry capped ghost ring (cycle/0004/v003) as the settled CLOCK-line shape. Drop newcomer-leniency duels, ghost-ring admission filters, and rings larger than 4. Do not spend another cycle on single-knob hit-rate tweaks on workload-a. The next cycle should get per-op-time and metadata-size recorded, or a second trace or cache size into the evaluation, and then compare the CLOCK+ghost and 1/8 LRU+ghost lines (and a 2- or 3-entry ring) on those axes. If the hit-rate search continues, the only lever left untested on the LRU line is ghost-hit insert depth.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Each variant took a different thread from Cycle 0006's continuation. v001 swapped the CLOCK-line ghost ring's stored keys for 16-bit fingerprints, a cost-only change. v002 inserted ghost hits on the LRU line at 3/8 depth instead of 1/4. v003 added a per-bucket miss-frequency sketch as a second admission signal on CLOCK. v001 won by matching its parent's hit rate exactly. H01 held on hit rate, but its real gain (about one-eighth of the ghost memory and a cheaper miss-path compare) still can't be seen, because the evaluation doesn't measure metadata-size or per-op-time. v002 slipped very slightly and v003 regressed clearly, so H02 and H03 were both refuted.

## Patterns

- The ghost ring only needs to answer "was this key evicted recently?" A membership test is enough, so a compact fingerprint ring loses nothing. This is the first change since Cycle 0004 that lowers cost on the settled CLOCK shape without moving hit rate.
- On LRU, ghost-hit insert depth peaks at 1/4. Going deeper gave nothing back, and the curve looks flat there. That was the last single-knob lever left on the LRU line. Depth, ring size, ring admission and insert depth have now all peaked at or before their current settings.
- The admission reward must stay rare. v003 kept the reward capped but widened the signal that grants it. Its small, shared, slow-ageing sketch saturated from hash collisions and ended up granting counter 1 to most newcomers, which in effect turned off the strict one-miss filter. Cycle 0004 found that a capped reward is necessary. This cycle shows it isn't sufficient: the trigger also has to stay selective.
- This is the second cycle running with no hit-rate gain on workload-a. The design family looks saturated on that trace. Seven cycles in, metadata-size and per-op-time are still unrecorded, and v001's win depends entirely on them.

## Open questions

- Would a miss-frequency signal with controlled error help? That would mean more counters, a short window, or a threshold that scales with the window. Or is the 4-entry exact ring already the complete admission signal for workload-a?
- Does fingerprinting the ghost ring work the same way on the LRU line?
- Once costs are measured, does the fingerprint ring (perhaps shrunk to 3 entries) make CLOCK+ghost the clear main line over LRU+ghost?

## Continuation

Adopt cycle/0007/v001 (CLOCK, strict newcomer filter, 4-entry fingerprint ghost ring) as the base going forward. Retire LRU-line insert-depth tuning and treat LRU+ghost as a reference line only. Drop broad frequency-based admission signals. Retry one only if its false-positive rate is bounded by design. Stop running hit-rate-only cycles on workload-a: getting metadata-size and per-op-time recorded, or a second trace or cache size added, is now required before any candidate can be told apart.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants tested a single knob on the cycle/0007/v001 CLOCK base (strict newcomer filter, 4-entry fingerprint ghost ring). v001 raised the first-hit promotion from 1 to 2. v002 recorded ghosts only for keys never hit while resident. v003 lowered the counter ceiling from 3 to 2. All three hypotheses were refuted, and each variant lost a small amount of hit rate. v002 is the listed winner only because its loss was the smallest. Its own insight says the ghost-admission filter gives no real leverage, so it adds no mechanism worth keeping.

## Patterns

- Hit-side protection now has a bracket on both sides. A bigger first-hit step (v001) and a lower ceiling (v003) both lost hit rate, so the uniform +1 step with a ceiling of 3 is locally optimal. Too much protection shields once-touched keys, and too little evicts keys that really are reused. Every GCLOCK knob has now been probed in both directions on workload-a.
- Ghost-ring admission filters lose on both lines, but for different reasons. On LRU (Cycle 0006), excluding never-hit keys hurt. On CLOCK, excluding hit keys hurt, though only barely: with the hand resting on the newcomer, most evictions are already never-hit keys, so the filter removes only a thin slice. Aged-out residents that were hit at some point are worth recording on both lines. The unfiltered 4-entry ring is already complete.
- One reward at the same small size works for both a ghost return and a first hit. Cycle 0004 found that the admission reward must be capped. This cycle finds the same cap applies on the hit side.
- This is the third cycle in a row with no hit-rate gain on workload-a, and the third in which the continuation asked for cost metrics that never arrived. v003's shorter worst-case sweep is a real cost saving that the evaluation still can't see.

## Open questions

- Would raising the ceiling *together with* a larger first-hit step help, so that the extra reward doesn't flatten the frequency range? It's untested, but the trend in this cohort suggests a small effect at best.
- Is ceiling 2 worth taking once per-op-time is measured? Its hit-rate loss is small and its worst-case sweep is a third shorter.
- Do any of the losses in the last three cycles exceed run-to-run noise on a second trace or cache size?

## Continuation

Treat cycle/0007/v001 as final for hit rate on workload-a. The design is fully bracketed: ring size, ring admission, newcomer grace, hit-step size and counter ceiling all peak at their current settings. Drop all single-knob CLOCK hit-rate probes, including ring-admission filters and hit-reward magnitude. The next cycle should not propose another hit-rate-only tweak on workload-a. It should get per-op-time and metadata-size recorded, or add a second trace or cache size, and then re-judge ceiling 2, a 3-entry fingerprint ring and the LRU+ghost reference line on those axes.


## Rejected variants

- v001 — gate:invariant gate (3/3 attempts)
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

## Write

Exactly one file: `cycle-insight.md` in the output dir.
