# Task: variant-implementer

Variant: v002
Parent variant: baseline (/home/user/cache-evolve/inputs/subjects/dash/baseline)
Hypothesis: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0006/hypotheses/H02.md
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0006/variants/v002/attempts/02/overlay (overlay-by-path)

## Paths

- Parent overlay/:       /home/user/cache-evolve/inputs/subjects/dash/baseline/overlay
- Parent fitness-vector: /home/user/cache-evolve/inputs/subjects/dash/baseline/eval/fitness-vector.json
- Parent insight.md:     /home/user/cache-evolve/inputs/subjects/dash/baseline/insight.md
- Subject src/:          /home/user/cache-evolve/inputs/subjects/dash/src
- Subject manifest:      /home/user/cache-evolve/inputs/subjects/dash/manifest.toml

## Authoring scope

full-autonomy

## Failing gate
The previous attempt's overlay was rejected. Diagnose and fix it:
- Previous overlay (the parent you are fixing): /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0006/variants/v002/attempts/01/overlay
- Failing gate's test scaffold: /home/user/cache-evolve/inputs/subjects/dash/tests/invariant_gate.rs

Latest rejection entry:

## 2026-06-05T19:06:41Z — variant=cycle/0006/v002 hypothesis=H02 attempt=1

**Gate**: invariant gate (harness error)
**Failing invariant**: <harness error — see Error block>
**Error**:
```

running 9 tests
test capacity_conservation_detects_inflated_geometry ... ok
test capacity_bound_negative_fixture ... ok
test capacity_conservation_holds_for_baseline_geometry ... ok
test single_eviction_per_put_negative_fixture ... ok
test no_phantom_items_negative_fixture ... FAILED
test slot_legality_negative_fixture ... FAILED
test stash_terminal_negative_fixture ... FAILED
test harness_tests::dash_invariants_runs_baseline_suite ... ok
test harness_tests::dash_segment_passes_all_invariants ... ok

failures:

---- no_phantom_items_negative_fixture stdout ----

thread 'no_phantom_items_negative_fixture' (24345) panicked at tests/invariant_gate.rs:41:5:
fixture: item in wrong bucket must be invisible to contains_key
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:80:14
   2: invariant_gate::no_phantom_items_negative_fixture
             at ./tests/invariant_gate.rs:41:2
   3: invariant_gate::no_phantom_items_negative_fixture::{{closure}}
             at ./tests/invariant_gate.rs:34:39
   4: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
   5: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.

---- slot_legality_negative_fixture stdout ----

thread 'slot_legality_negative_fixture' (24347) panicked at tests/invariant_gate.rs:74:5:
slot_legality check must catch item in wrong bucket
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:80:14
   2: invariant_gate::slot_legality_negative_fixture
             at ./tests/invariant_gate.rs:74:2
   3: invariant_gate::slot_legality_negative_fixture::{{closure}}
             at ./tests/invariant_gate.rs:68:36
   4: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
   5: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.

---- stash_terminal_negative_fixture stdout ----

thread 'stash_terminal_negative_fixture' (24348) panicked at tests/invariant_gate.rs:110:5:
fixture: ghost item must be invisible to contains_key
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:80:14
   2: invariant_gate::stash_terminal_negative_fixture
             at ./tests/invariant_gate.rs:110:2
   3: invariant_gate::stash_terminal_negative_fixture::{{closure}}
             at ./tests/invariant_gate.rs:100:37
   4: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
   5: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.


failures:
    no_phantom_items_negative_fixture
    slot_legality_negative_fixture
    stash_terminal_negative_fixture

test result: FAILED. 6 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s

    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/invariant_gate.rs (/tmp/variant-workspace-zztgtjxy/target/debug/deps/invariant_gate-7c505ed1f391d40d)
error: test failed, to rerun pass `--test invariant_gate`

```
---

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied verbatim to a copy of the subject crate at build time.
