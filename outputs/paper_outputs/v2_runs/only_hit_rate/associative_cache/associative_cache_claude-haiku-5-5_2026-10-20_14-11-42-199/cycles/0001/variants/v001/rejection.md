
## 2026-10-20T14:13:21Z — variant=cycle/0001/v001 hypothesis=H01 attempt=1

**Gate**: contract gate
**Error**:
```

running 1 test
test contract_suite ... FAILED

failures:

---- contract_suite stdout ----

thread 'contract_suite' (13100) panicked at src/shared/geometry.rs:42:9:
wide-bucket geometry requires total_capacity divisible by 64, got 8
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/panicking.rs:80:14
   2: <associative_cache::shared::geometry::BaselineGeometry as associative_cache::shared::traits::geometry::BucketGeometry>::layout
             at ./src/shared/geometry.rs:42:3
   3: associative_cache::factory::resolve_geometry
             at ./src/factory.rs:24:79
   4: associative_cache::factory::make_cache
             at ./src/factory.rs:47:17
   5: contract_gate::contract_suite::{closure#0}
             at ./tests/contract_gate.rs:20:24
   6: framework::test_suite::run_contract_suite::<contract_gate::contract_suite::{closure#0}>
             at /home/user/cache-evolve/framework-rs/src/test_suite.rs:22:15
   7: contract_gate::contract_suite
             at ./tests/contract_gate.rs:20:2
   8: contract_gate::contract_suite::{closure#0}
             at ./tests/contract_gate.rs:19:20
   9: <contract_gate::contract_suite::{closure#0} as core::ops::function::FnOnce<()>>::call_once
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ops/function.rs:250:5
  10: <fn() -> core::result::Result<(), alloc::string::String> as core::ops::function::FnOnce<()>>::call_once
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.


failures:
    contract_suite

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s

    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.23s
     Running tests/contract_gate.rs (/tmp/variant-workspace-2ygs3994/target/debug/deps/contract_gate-31cff4ae5d895760)
error: test failed, to rerun pass `--test contract_gate`

```
---
