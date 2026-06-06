# Task: variant-implementer

Variant: v003
Parent variant: baseline (/home/user/cache-evolve/inputs/subjects/dash/baseline)
Hypothesis: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0010/hypotheses/H03.md
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0010/variants/v003/attempts/02/overlay (overlay-by-path)

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
- Previous overlay (the parent you are fixing): /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0010/variants/v003/attempts/01/overlay
- Failing gate's test scaffold: /home/user/cache-evolve/inputs/subjects/dash/tests/contract_gate.rs

Latest rejection entry:

## 2026-06-05T20:16:13Z — variant=cycle/0010/v003 hypothesis=H03 attempt=1

**Gate**: contract gate
**Error**:
```

running 1 test
test contract_suite ... FAILED

failures:

---- contract_suite stdout ----

thread 'contract_suite' (14742) panicked at /home/user/cache-evolve/framework-rs/src/test_suite.rs:30:4:
contract gate: get(&1) returned None immediately after put(1, 100)
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:80:14
   2: core::panicking::panic_display
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:259:5
   3: core::option::expect_failed
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/option.rs:2208:5
   4: core::option::Option<T>::expect
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/option.rs:971:21
   5: framework::test_suite::run_contract_suite
             at /home/user/cache-evolve/framework-rs/src/test_suite.rs:30:4
   6: contract_gate::contract_suite
             at ./tests/contract_gate.rs:23:2
   7: contract_gate::contract_suite::{{closure}}
             at ./tests/contract_gate.rs:22:20
   8: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
   9: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.


failures:
    contract_suite

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/contract_gate.rs (/tmp/variant-workspace-_eehc8yk/target/debug/deps/contract_gate-91e7803692e93999)
error: test failed, to rerun pass `--test contract_gate`

```
---

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied verbatim to a copy of the subject crate at build time.
