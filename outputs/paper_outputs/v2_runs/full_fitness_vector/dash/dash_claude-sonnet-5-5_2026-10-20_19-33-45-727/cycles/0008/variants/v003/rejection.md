
## 2026-10-20T20:11:49Z — variant=cycle/0008/v003 hypothesis=H03 attempt=1

**Gate**: contract gate
**Error**:
```

running 1 test
test contract_suite ... FAILED

failures:

---- contract_suite stdout ----

thread 'contract_suite' (8545) panicked at /home/user/cache-evolve/framework-rs/src/test_suite.rs:30:4:
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

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.10s
     Running tests/contract_gate.rs (/tmp/variant-workspace-wrnzln4h/target/debug/deps/contract_gate-1d0a194e9eced445)
error: test failed, to rerun pass `--test contract_gate`

```
---
