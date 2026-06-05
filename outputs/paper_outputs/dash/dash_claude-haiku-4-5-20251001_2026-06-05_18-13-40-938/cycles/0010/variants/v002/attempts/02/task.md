# Task: variant-implementer

Variant: v002
Parent variant: baseline (/home/user/cache-evolve/inputs/subjects/dash/baseline)
Hypothesis: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0010/hypotheses/H02.md
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0010/variants/v002/attempts/02/overlay (overlay-by-path)

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
- Previous overlay (the parent you are fixing): /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0010/variants/v002/attempts/01/overlay
- Failing gate's test scaffold: /home/user/cache-evolve/inputs/subjects/dash/tests/contract_gate.rs

Latest rejection entry:

## 2026-06-05T19:49:16Z — variant=cycle/0010/v002 hypothesis=H02 attempt=1

**Gate**: contract gate
**Error**:
```

running 1 test
test contract_suite ... FAILED

failures:

---- contract_suite stdout ----

thread 'contract_suite' (21733) panicked at src/factory.rs:48:10:
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

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/contract_gate.rs (/tmp/variant-workspace-ae289drq/target/debug/deps/contract_gate-91e7803692e93999)
error: test failed, to rerun pass `--test contract_gate`

```
---

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied verbatim to a copy of the subject crate at build time.
