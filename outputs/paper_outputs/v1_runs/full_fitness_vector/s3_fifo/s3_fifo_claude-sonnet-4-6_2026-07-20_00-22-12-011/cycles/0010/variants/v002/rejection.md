
## 2026-07-21T10:00:21Z — variant=cycle/0010/v002 hypothesis=H02 attempt=1

**Gate**: contract gate
**Error**:
```

running 1 test
test contract_suite ... FAILED

failures:

---- contract_suite stdout ----

thread 'contract_suite' (22740) panicked at C:\Users\yaniv\Projects\cache-evolve\framework-rs\src\test_suite.rs:30:4:
contract gate: get(&1) returned None immediately after put(1, 100)
stack backtrace:
   0: std::panicking::panic_handler
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library\std\src\panicking.rs:689
   1: core::panicking::panic_fmt
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library\core\src\panicking.rs:80
   2: core::panicking::panic_display
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library\core\src\panicking.rs:259
   3: core::option::expect_failed
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library\core\src\option.rs:2257
   4: enum2$<core::option::Option<i64> >::expect<i64>
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library\core\src\option.rs:968
   5: framework::test_suite::run_contract_suite<contract_gate::contract_suite::closure_env$0>
             at C:\Users\yaniv\Projects\cache-evolve\framework-rs\src\test_suite.rs:30
   6: contract_gate::contract_suite
             at .\tests\contract_gate.rs:11
   7: contract_gate::contract_suite::closure$0
             at .\tests\contract_gate.rs:10
   8: core::ops::function::FnOnce::call_once<contract_gate::contract_suite::closure_env$0,tuple$<> >
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library\core\src\ops\function.rs:250
   9: core::ops::function::FnOnce::call_once
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library\core\src\ops\function.rs:250
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.


failures:
    contract_suite

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

warning: associated function `new` is never used
  --> src\s3_fifo\mod.rs:40:8
   |
39 | impl<K, V> Entry<K, V> {
   | ---------------------- associated function in this implementation
40 |     fn new(key: K, _value: V) -> Self {
   |        ^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `s3_fifo` (lib) generated 1 warning
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.16s
     Running tests\contract_gate.rs (C:\Users\yaniv\AppData\Local\Temp\variant-workspace-aewiquuy\target\debug\deps\contract_gate-7cb25797759f9344.exe)
error: test failed, to rerun pass `--test contract_gate`

```
---
