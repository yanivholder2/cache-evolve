# Task: variant-implementer

Variant: v003
Parent variant: cycle/0004/v002 (/home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0004/variants/v002)
Hypothesis: /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0005/hypotheses/H03.md
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0005/variants/v003/attempts/03/overlay (overlay-by-path)

## Paths

- Parent overlay/:       /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0004/variants/v002/attempts/01/overlay
- Parent fitness-vector: /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0004/variants/v002/eval/fitness-vector.json
- Parent insight.md:     /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0004/variants/v002/insight.md
- Subject src/:          /home/user/cache-evolve/inputs/subjects/dash/src
- Subject manifest:      /home/user/cache-evolve/inputs/subjects/dash/manifest.toml

## Authoring scope

full-autonomy

Gate-integrity files are pinned to the pristine crate and may not be
authored in any scope — overlay copies of them are ignored and rejected:
src/dash/jni.rs, src/factory.rs, Cargo.toml, src/lib.rs, tests/, src/test_utils/, src/bin/

## Failing gate
The previous attempt's overlay was rejected. Diagnose and fix it:
- Previous overlay (the parent you are fixing): /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0005/variants/v003/attempts/02/overlay
- Failing gate's test scaffold: /home/user/cache-evolve/inputs/subjects/dash/tests/contract_gate.rs

Latest rejection entry:

## 2026-10-21T04:25:51Z — variant=cycle/0005/v003 hypothesis=H03 attempt=2

**Gate**: contract gate
**Error**:
```

running 1 test
test contract_suite ... FAILED

failures:

---- contract_suite stdout ----

thread 'contract_suite' (5683) panicked at /home/user/cache-evolve/framework-rs/src/test_suite.rs:30:4:
contract gate: get(&1) returned None immediately after put(1, 100)
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/panicking.rs:80:14
   2: core::panicking::panic_display::<&str>
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/panicking.rs:259:5
   3: core::option::expect_failed
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/option.rs:2257:5
   4: <core::option::Option<i64>>::expect
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/option.rs:968:21
   5: framework::test_suite::run_contract_suite::<contract_gate::contract_suite::{closure#0}>
             at /home/user/cache-evolve/framework-rs/src/test_suite.rs:30:4
   6: contract_gate::contract_suite
             at ./tests/contract_gate.rs:23:2
   7: contract_gate::contract_suite::{closure#0}
             at ./tests/contract_gate.rs:22:20
   8: <contract_gate::contract_suite::{closure#0} as core::ops::function::FnOnce<()>>::call_once
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ops/function.rs:250:5
   9: <fn() -> core::result::Result<(), alloc::string::String> as core::ops::function::FnOnce<()>>::call_once
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.


failures:
    contract_suite

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s

    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running tests/contract_gate.rs (/tmp/variant-workspace-s9sq3z4e/target/debug/deps/contract_gate-1d0a194e9eced445)
error: test failed, to rerun pass `--test contract_gate`

```
---

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied to a copy of the subject crate at build time (gate-integrity
files excluded).
