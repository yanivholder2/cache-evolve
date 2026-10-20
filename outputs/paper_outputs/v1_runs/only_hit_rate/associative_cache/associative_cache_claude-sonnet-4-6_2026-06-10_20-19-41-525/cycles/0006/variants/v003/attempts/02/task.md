# Task: variant-implementer

Variant: v003
Parent variant: baseline (/home/user/cache-evolve/inputs/subjects/associative_cache/baseline)
Hypothesis: /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0006/hypotheses/H03.md
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0006/variants/v003/attempts/02/overlay (overlay-by-path)

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
- Previous overlay (the parent you are fixing): /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0006/variants/v003/attempts/01/overlay
- Failing gate's test scaffold: /home/user/cache-evolve/inputs/subjects/associative_cache/tests/invariant_gate.rs

Latest rejection entry:

## 2026-06-10T21:31:46Z — variant=cycle/0006/v003 hypothesis=H03 attempt=1

**Gate**: invariant gate (harness error)
**Failing invariant**: <harness error — see Error block>
**Error**:
```

running 6 tests
test capacity_conservation_detects_inflated_geometry ... ok
test capacity_bound_holds_under_overfill ... ok
test capacity_conservation_holds_for_baseline_geometry ... ok
test slot_legality_holds_under_workload ... ok
test associative_invariants_runs_baseline_suite ... FAILED
test baseline_passes_all_invariants ... FAILED

failures:

---- associative_invariants_runs_baseline_suite stdout ----
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known

thread 'associative_invariants_runs_baseline_suite' (24741) panicked at src/shared/geometry.rs:36:9:
assertion `left == right` failed: total capacity 2159 must be divisible by target_num_buckets 4
  left: 3
 right: 0
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:80:14
   2: core::panicking::assert_failed_inner
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:434:23
   3: core::panicking::assert_failed
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:394:5
   4: associative_cache::shared::geometry::BaselineGeometry::for_settings
             at ./src/shared/geometry.rs:36:9
   5: associative_cache::factory::resolve_geometry
             at ./src/factory.rs:24:38
   6: associative_cache::test_utils::invariants::run_invariant_suite::{{closure}}
             at ./src/test_utils/invariants.rs:133:17
   7: core::ops::function::impls::<impl core::ops::function::Fn<A> for &F>::call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:263:22
   8: proptest::test_runner::runner::call_test::{{closure}}::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:56
   9: <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panic/unwind_safe.rs:274:9
  10: std::panicking::catch_unwind::do_call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:581:40
  11: __rust_try
  12: std::panicking::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:544:19
  13: std::panic::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panic.rs:359:14
  14: proptest::test_runner::runner::call_test::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:16
  15: proptest::test_runner::scoped_panic_hook::internal::with_hook
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/scoped_panic_hook.rs:130:9
  16: proptest::test_runner::runner::call_test
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:256:9
  17: proptest::test_runner::runner::TestRunner::run_one_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:729:22
  18: proptest::test_runner::runner::TestRunner::gen_and_run_case
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:671:28
  19: proptest::test_runner::runner::TestRunner::run_in_process_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:620:31
  20: proptest::test_runner::runner::TestRunner::run_in_process
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:575:14
  21: proptest::test_runner::runner::TestRunner::run
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:418:18
  22: associative_cache::test_utils::invariants::run_invariant_suite
             at ./src/test_utils/invariants.rs:132:4
  23: <associative_cache::test_utils::invariants::AssociativeCacheInvariants as framework::Invariants>::run_invariant_suite
             at ./src/test_utils/invariants.rs:29:3
  24: invariant_gate::associative_invariants_runs_baseline_suite
             at ./tests/invariant_gate.rs:98:2
  25: invariant_gate::associative_invariants_runs_baseline_suite::{{closure}}
             at ./tests/invariant_gate.rs:96:48
  26: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
  27: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.

thread 'associative_invariants_runs_baseline_suite' (24741) panicked at src/shared/geometry.rs:36:9:
assertion `left == right` failed: total capacity 1143 must be divisible by target_num_buckets 2
  left: 1
 right: 0
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:80:14
   2: core::panicking::assert_failed_inner
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:434:23
   3: core::panicking::assert_failed
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:394:5
   4: associative_cache::shared::geometry::BaselineGeometry::for_settings
             at ./src/shared/geometry.rs:36:9
   5: associative_cache::factory::resolve_geometry
             at ./src/factory.rs:24:38
   6: associative_cache::test_utils::invariants::run_invariant_suite::{{closure}}
             at ./src/test_utils/invariants.rs:133:17
   7: core::ops::function::impls::<impl core::ops::function::Fn<A> for &F>::call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:263:22
   8: proptest::test_runner::runner::call_test::{{closure}}::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:56
   9: <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panic/unwind_safe.rs:274:9
  10: std::panicking::catch_unwind::do_call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:581:40
  11: __rust_try
  12: std::panicking::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:544:19
  13: std::panic::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panic.rs:359:14
  14: proptest::test_runner::runner::call_test::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:16
  15: proptest::test_runner::scoped_panic_hook::internal::with_hook
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/scoped_panic_hook.rs:130:9
  16: proptest::test_runner::runner::call_test
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:256:9
  17: proptest::test_runner::runner::TestRunner::shrink
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:856:30
  18: proptest::test_runner::runner::TestRunner::run_one_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:743:22
  19: proptest::test_runner::runner::TestRunner::gen_and_run_case
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:671:28
  20: proptest::test_runner::runner::TestRunner::run_in_process_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:620:31
  21: proptest::test_runner::runner::TestRunner::run_in_process
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:575:14
  22: proptest::test_runner::runner::TestRunner::run
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:418:18
  23: associative_cache::test_utils::invariants::run_invariant_suite
             at ./src/test_utils/invariants.rs:132:4
  24: <associative_cache::test_utils::invariants::AssociativeCacheInvariants as framework::Invariants>::run_invariant_suite
             at ./src/test_utils/invariants.rs:29:3
  25: invariant_gate::associative_invariants_runs_baseline_suite
             at ./tests/invariant_gate.rs:98:2
  26: invariant_gate::associative_invariants_runs_baseline_suite::{{closure}}
             at ./tests/invariant_gate.rs:96:48
  27: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
  28: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.

thread 'associative_invariants_runs_baseline_suite' (24741) panicked at src/shared/geometry.rs:36:9:
assertion `left == right` failed: total capacity 1143 must be divisible by target_num_buckets 2
  left: 1
 right: 0
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:80:14
   2: core::panicking::assert_failed_inner
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:434:23
   3: core::panicking::assert_failed
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:394:5
   4: associative_cache::shared::geometry::BaselineGeometry::for_settings
             at ./src/shared/geometry.rs:36:9
   5: associative_cache::factory::resolve_geometry
             at ./src/factory.rs:24:38
   6: associative_cache::test_utils::invariants::run_invariant_suite::{{closure}}
             at ./src/test_utils/invariants.rs:133:17
   7: core::ops::function::impls::<impl core::ops::function::Fn<A> for &F>::call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:263:22
   8: proptest::test_runner::runner::call_test::{{closure}}::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:56
   9: <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panic/unwind_safe.rs:274:9
  10: std::panicking::catch_unwind::do_call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:581:40
  11: __rust_try
  12: std::panicking::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:544:19
  13: std::panic::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panic.rs:359:14
  14: proptest::test_runner::runner::call_test::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:16
  15: proptest::test_runner::scoped_panic_hook::internal::with_hook
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/scoped_panic_hook.rs:130:9
  16: proptest::test_runner::runner::call_test
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:256:9
  17: proptest::test_runner::runner::TestRunner::shrink
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:856:30
  18: proptest::test_runner::runner::TestRunner::run_one_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:743:22
  19: proptest::test_runner::runner::TestRunner::gen_and_run_case
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:671:28
  20: proptest::test_runner::runner::TestRunner::run_in_process_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:620:31
  21: proptest::test_runner::runner::TestRunner::run_in_process
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:575:14
  22: proptest::test_runner::runner::TestRunner::run
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:418:18
  23: associative_cache::test_utils::invariants::run_invariant_suite
             at ./src/test_utils/invariants.rs:132:4
  24: <associative_cache::test_utils::invariants::AssociativeCacheInvariants as framework::Invariants>::run_invariant_suite
             at ./src/test_utils/invariants.rs:29:3
  25: invariant_gate::associative_invariants_runs_baseline_suite
             at ./tests/invariant_gate.rs:98:2
  26: invariant_gate::associative_invariants_runs_baseline_suite::{{closure}}
             at ./tests/invariant_gate.rs:96:48
  27: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
  28: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.

thread 'associative_invariants_runs_baseline_suite' (24741) panicked at src/shared/geometry.rs:36:9:
assertion `left == right` failed: total capacity 1143 must be divisible by target_num_buckets 2
  left: 1
 right: 0
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:80:14
   2: core::panicking::assert_failed_inner
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:434:23
   3: core::panicking::assert_failed
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:394:5
   4: associative_cache::shared::geometry::BaselineGeometry::for_settings
             at ./src/shared/geometry.rs:36:9
   5: associative_cache::factory::resolve_geometry
             at ./src/factory.rs:24:38
   6: associative_cache::test_utils::invariants::run_invariant_suite::{{closure}}
             at ./src/test_utils/invariants.rs:133:17
   7: core::ops::function::impls::<impl core::ops::function::Fn<A> for &F>::call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:263:22
   8: proptest::test_runner::runner::call_test::{{closure}}::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:56
   9: <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panic/unwind_safe.rs:274:9
  10: std::panicking::catch_unwind::do_call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:581:40
  11: __rust_try
  12: std::panicking::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:544:19
  13: std::panic::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panic.rs:359:14
  14: proptest::test_runner::runner::call_test::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:16
  15: proptest::test_runner::scoped_panic_hook::internal::with_hook
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/scoped_panic_hook.rs:130:9
  16: proptest::test_runner::runner::call_test
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:256:9
  17: proptest::test_runner::runner::TestRunner::shrink
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:856:30
  18: proptest::test_runner::runner::TestRunner::run_one_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:743:22
  19: proptest::test_runner::runner::TestRunner::gen_and_run_case
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:671:28
  20: proptest::test_runner::runner::TestRunner::run_in_process_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:620:31
  21: proptest::test_runner::runner::TestRunner::run_in_process
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:575:14
  22: proptest::test_runner::runner::TestRunner::run
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:418:18
  23: associative_cache::test_utils::invariants::run_invariant_suite
             at ./src/test_utils/invariants.rs:132:4
  24: <associative_cache::test_utils::invariants::AssociativeCacheInvariants as framework::Invariants>::run_invariant_suite
             at ./src/test_utils/invariants.rs:29:3
  25: invariant_gate::associative_invariants_runs_baseline_suite
             at ./tests/invariant_gate.rs:98:2
  26: invariant_gate::associative_invariants_runs_baseline_suite::{{closure}}
             at ./tests/invariant_gate.rs:96:48
  27: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
  28: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.
proptest: FileFailurePersistence::SourceParallel set, but no source file known

thread 'associative_invariants_runs_baseline_suite' (24741) panicked at tests/invariant_gate.rs:98:55:
called `Result::unwrap()` on an `Err` value: "P4 capacity_conservation: Test failed: assertion `left == right` failed: total capacity 1143 must be divisible by target_num_buckets 2\n  left: 1\n right: 0.\nminimal failing input: AssociativeCacheSettings {\n    num_of_buckets: 9,\n    bucket_size: 127,\n    eviction_policy: ClassicLRU,\n}"
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:80:14
   2: core::result::unwrap_failed
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/result.rs:1867:5
   3: core::result::Result<T,E>::unwrap
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/result.rs:1233:23
   4: invariant_gate::associative_invariants_runs_baseline_suite
             at ./tests/invariant_gate.rs:98:52
   5: invariant_gate::associative_invariants_runs_baseline_suite::{{closure}}
             at ./tests/invariant_gate.rs:96:48
   6: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
   7: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.

---- baseline_passes_all_invariants stdout ----
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known
proptest: FileFailurePersistence::SourceParallel set, but no source file known

thread 'baseline_passes_all_invariants' (24742) panicked at src/shared/geometry.rs:36:9:
assertion `left == right` failed: total capacity 2079 must be divisible by target_num_buckets 6
  left: 3
 right: 0
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:80:14
   2: core::panicking::assert_failed_inner
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:434:23
   3: core::panicking::assert_failed
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:394:5
   4: associative_cache::shared::geometry::BaselineGeometry::for_settings
             at ./src/shared/geometry.rs:36:9
   5: associative_cache::factory::resolve_geometry
             at ./src/factory.rs:24:38
   6: associative_cache::test_utils::invariants::run_invariant_suite::{{closure}}
             at ./src/test_utils/invariants.rs:133:17
   7: core::ops::function::impls::<impl core::ops::function::Fn<A> for &F>::call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:263:22
   8: proptest::test_runner::runner::call_test::{{closure}}::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:56
   9: <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panic/unwind_safe.rs:274:9
  10: std::panicking::catch_unwind::do_call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:581:40
  11: __rust_try
  12: std::panicking::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:544:19
  13: std::panic::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panic.rs:359:14
  14: proptest::test_runner::runner::call_test::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:16
  15: proptest::test_runner::scoped_panic_hook::internal::with_hook
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/scoped_panic_hook.rs:130:9
  16: proptest::test_runner::runner::call_test
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:256:9
  17: proptest::test_runner::runner::TestRunner::run_one_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:729:22
  18: proptest::test_runner::runner::TestRunner::gen_and_run_case
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:671:28
  19: proptest::test_runner::runner::TestRunner::run_in_process_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:620:31
  20: proptest::test_runner::runner::TestRunner::run_in_process
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:575:14
  21: proptest::test_runner::runner::TestRunner::run
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:418:18
  22: associative_cache::test_utils::invariants::run_invariant_suite
             at ./src/test_utils/invariants.rs:132:4
  23: invariant_gate::baseline_passes_all_invariants
             at ./tests/invariant_gate.rs:92:2
  24: invariant_gate::baseline_passes_all_invariants::{{closure}}
             at ./tests/invariant_gate.rs:91:36
  25: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
  26: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.

thread 'baseline_passes_all_invariants' (24742) panicked at src/shared/geometry.rs:36:9:
assertion `left == right` failed: total capacity 1078 must be divisible by target_num_buckets 3
  left: 1
 right: 0
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:80:14
   2: core::panicking::assert_failed_inner
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:434:23
   3: core::panicking::assert_failed
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:394:5
   4: associative_cache::shared::geometry::BaselineGeometry::for_settings
             at ./src/shared/geometry.rs:36:9
   5: associative_cache::factory::resolve_geometry
             at ./src/factory.rs:24:38
   6: associative_cache::test_utils::invariants::run_invariant_suite::{{closure}}
             at ./src/test_utils/invariants.rs:133:17
   7: core::ops::function::impls::<impl core::ops::function::Fn<A> for &F>::call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:263:22
   8: proptest::test_runner::runner::call_test::{{closure}}::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:56
   9: <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panic/unwind_safe.rs:274:9
  10: std::panicking::catch_unwind::do_call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:581:40
  11: __rust_try
  12: std::panicking::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:544:19
  13: std::panic::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panic.rs:359:14
  14: proptest::test_runner::runner::call_test::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:16
  15: proptest::test_runner::scoped_panic_hook::internal::with_hook
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/scoped_panic_hook.rs:130:9
  16: proptest::test_runner::runner::call_test
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:256:9
  17: proptest::test_runner::runner::TestRunner::shrink
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:856:30
  18: proptest::test_runner::runner::TestRunner::run_one_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:743:22
  19: proptest::test_runner::runner::TestRunner::gen_and_run_case
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:671:28
  20: proptest::test_runner::runner::TestRunner::run_in_process_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:620:31
  21: proptest::test_runner::runner::TestRunner::run_in_process
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:575:14
  22: proptest::test_runner::runner::TestRunner::run
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:418:18
  23: associative_cache::test_utils::invariants::run_invariant_suite
             at ./src/test_utils/invariants.rs:132:4
  24: invariant_gate::baseline_passes_all_invariants
             at ./tests/invariant_gate.rs:92:2
  25: invariant_gate::baseline_passes_all_invariants::{{closure}}
             at ./tests/invariant_gate.rs:91:36
  26: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
  27: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.

thread 'baseline_passes_all_invariants' (24742) panicked at src/shared/geometry.rs:36:9:
assertion `left == right` failed: total capacity 847 must be divisible by target_num_buckets 2
  left: 1
 right: 0
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:80:14
   2: core::panicking::assert_failed_inner
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:434:23
   3: core::panicking::assert_failed
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:394:5
   4: associative_cache::shared::geometry::BaselineGeometry::for_settings
             at ./src/shared/geometry.rs:36:9
   5: associative_cache::factory::resolve_geometry
             at ./src/factory.rs:24:38
   6: associative_cache::test_utils::invariants::run_invariant_suite::{{closure}}
             at ./src/test_utils/invariants.rs:133:17
   7: core::ops::function::impls::<impl core::ops::function::Fn<A> for &F>::call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:263:22
   8: proptest::test_runner::runner::call_test::{{closure}}::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:56
   9: <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panic/unwind_safe.rs:274:9
  10: std::panicking::catch_unwind::do_call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:581:40
  11: __rust_try
  12: std::panicking::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:544:19
  13: std::panic::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panic.rs:359:14
  14: proptest::test_runner::runner::call_test::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:16
  15: proptest::test_runner::scoped_panic_hook::internal::with_hook
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/scoped_panic_hook.rs:130:9
  16: proptest::test_runner::runner::call_test
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:256:9
  17: proptest::test_runner::runner::TestRunner::shrink
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:856:30
  18: proptest::test_runner::runner::TestRunner::run_one_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:743:22
  19: proptest::test_runner::runner::TestRunner::gen_and_run_case
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:671:28
  20: proptest::test_runner::runner::TestRunner::run_in_process_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:620:31
  21: proptest::test_runner::runner::TestRunner::run_in_process
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:575:14
  22: proptest::test_runner::runner::TestRunner::run
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:418:18
  23: associative_cache::test_utils::invariants::run_invariant_suite
             at ./src/test_utils/invariants.rs:132:4
  24: invariant_gate::baseline_passes_all_invariants
             at ./tests/invariant_gate.rs:92:2
  25: invariant_gate::baseline_passes_all_invariants::{{closure}}
             at ./tests/invariant_gate.rs:91:36
  26: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
  27: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.

thread 'baseline_passes_all_invariants' (24742) panicked at src/shared/geometry.rs:36:9:
assertion `left == right` failed: total capacity 693 must be divisible by target_num_buckets 2
  left: 1
 right: 0
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:80:14
   2: core::panicking::assert_failed_inner
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:434:23
   3: core::panicking::assert_failed
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:394:5
   4: associative_cache::shared::geometry::BaselineGeometry::for_settings
             at ./src/shared/geometry.rs:36:9
   5: associative_cache::factory::resolve_geometry
             at ./src/factory.rs:24:38
   6: associative_cache::test_utils::invariants::run_invariant_suite::{{closure}}
             at ./src/test_utils/invariants.rs:133:17
   7: core::ops::function::impls::<impl core::ops::function::Fn<A> for &F>::call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:263:22
   8: proptest::test_runner::runner::call_test::{{closure}}::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:56
   9: <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panic/unwind_safe.rs:274:9
  10: std::panicking::catch_unwind::do_call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:581:40
  11: __rust_try
  12: std::panicking::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:544:19
  13: std::panic::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panic.rs:359:14
  14: proptest::test_runner::runner::call_test::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:16
  15: proptest::test_runner::scoped_panic_hook::internal::with_hook
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/scoped_panic_hook.rs:130:9
  16: proptest::test_runner::runner::call_test
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:256:9
  17: proptest::test_runner::runner::TestRunner::shrink
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:856:30
  18: proptest::test_runner::runner::TestRunner::run_one_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:743:22
  19: proptest::test_runner::runner::TestRunner::gen_and_run_case
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:671:28
  20: proptest::test_runner::runner::TestRunner::run_in_process_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:620:31
  21: proptest::test_runner::runner::TestRunner::run_in_process
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:575:14
  22: proptest::test_runner::runner::TestRunner::run
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:418:18
  23: associative_cache::test_utils::invariants::run_invariant_suite
             at ./src/test_utils/invariants.rs:132:4
  24: invariant_gate::baseline_passes_all_invariants
             at ./tests/invariant_gate.rs:92:2
  25: invariant_gate::baseline_passes_all_invariants::{{closure}}
             at ./tests/invariant_gate.rs:91:36
  26: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
  27: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.

thread 'baseline_passes_all_invariants' (24742) panicked at src/shared/geometry.rs:36:9:
assertion `left == right` failed: total capacity 693 must be divisible by target_num_buckets 2
  left: 1
 right: 0
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:80:14
   2: core::panicking::assert_failed_inner
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:434:23
   3: core::panicking::assert_failed
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:394:5
   4: associative_cache::shared::geometry::BaselineGeometry::for_settings
             at ./src/shared/geometry.rs:36:9
   5: associative_cache::factory::resolve_geometry
             at ./src/factory.rs:24:38
   6: associative_cache::test_utils::invariants::run_invariant_suite::{{closure}}
             at ./src/test_utils/invariants.rs:133:17
   7: core::ops::function::impls::<impl core::ops::function::Fn<A> for &F>::call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:263:22
   8: proptest::test_runner::runner::call_test::{{closure}}::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:56
   9: <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panic/unwind_safe.rs:274:9
  10: std::panicking::catch_unwind::do_call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:581:40
  11: __rust_try
  12: std::panicking::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:544:19
  13: std::panic::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panic.rs:359:14
  14: proptest::test_runner::runner::call_test::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:16
  15: proptest::test_runner::scoped_panic_hook::internal::with_hook
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/scoped_panic_hook.rs:130:9
  16: proptest::test_runner::runner::call_test
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:256:9
  17: proptest::test_runner::runner::TestRunner::shrink
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:856:30
  18: proptest::test_runner::runner::TestRunner::run_one_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:743:22
  19: proptest::test_runner::runner::TestRunner::gen_and_run_case
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:671:28
  20: proptest::test_runner::runner::TestRunner::run_in_process_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:620:31
  21: proptest::test_runner::runner::TestRunner::run_in_process
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:575:14
  22: proptest::test_runner::runner::TestRunner::run
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:418:18
  23: associative_cache::test_utils::invariants::run_invariant_suite
             at ./src/test_utils/invariants.rs:132:4
  24: invariant_gate::baseline_passes_all_invariants
             at ./tests/invariant_gate.rs:92:2
  25: invariant_gate::baseline_passes_all_invariants::{{closure}}
             at ./tests/invariant_gate.rs:91:36
  26: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
  27: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.

thread 'baseline_passes_all_invariants' (24742) panicked at src/shared/geometry.rs:36:9:
assertion `left == right` failed: total capacity 351 must be divisible by target_num_buckets 2
  left: 1
 right: 0
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:80:14
   2: core::panicking::assert_failed_inner
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:434:23
   3: core::panicking::assert_failed
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:394:5
   4: associative_cache::shared::geometry::BaselineGeometry::for_settings
             at ./src/shared/geometry.rs:36:9
   5: associative_cache::factory::resolve_geometry
             at ./src/factory.rs:24:38
   6: associative_cache::test_utils::invariants::run_invariant_suite::{{closure}}
             at ./src/test_utils/invariants.rs:133:17
   7: core::ops::function::impls::<impl core::ops::function::Fn<A> for &F>::call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:263:22
   8: proptest::test_runner::runner::call_test::{{closure}}::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:56
   9: <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panic/unwind_safe.rs:274:9
  10: std::panicking::catch_unwind::do_call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:581:40
  11: __rust_try
  12: std::panicking::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:544:19
  13: std::panic::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panic.rs:359:14
  14: proptest::test_runner::runner::call_test::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:16
  15: proptest::test_runner::scoped_panic_hook::internal::with_hook
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/scoped_panic_hook.rs:130:9
  16: proptest::test_runner::runner::call_test
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:256:9
  17: proptest::test_runner::runner::TestRunner::shrink
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:856:30
  18: proptest::test_runner::runner::TestRunner::run_one_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:743:22
  19: proptest::test_runner::runner::TestRunner::gen_and_run_case
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:671:28
  20: proptest::test_runner::runner::TestRunner::run_in_process_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:620:31
  21: proptest::test_runner::runner::TestRunner::run_in_process
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:575:14
  22: proptest::test_runner::runner::TestRunner::run
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:418:18
  23: associative_cache::test_utils::invariants::run_invariant_suite
             at ./src/test_utils/invariants.rs:132:4
  24: invariant_gate::baseline_passes_all_invariants
             at ./tests/invariant_gate.rs:92:2
  25: invariant_gate::baseline_passes_all_invariants::{{closure}}
             at ./tests/invariant_gate.rs:91:36
  26: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
  27: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.

thread 'baseline_passes_all_invariants' (24742) panicked at src/shared/geometry.rs:36:9:
assertion `left == right` failed: total capacity 315 must be divisible by target_num_buckets 2
  left: 1
 right: 0
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:80:14
   2: core::panicking::assert_failed_inner
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:434:23
   3: core::panicking::assert_failed
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:394:5
   4: associative_cache::shared::geometry::BaselineGeometry::for_settings
             at ./src/shared/geometry.rs:36:9
   5: associative_cache::factory::resolve_geometry
             at ./src/factory.rs:24:38
   6: associative_cache::test_utils::invariants::run_invariant_suite::{{closure}}
             at ./src/test_utils/invariants.rs:133:17
   7: core::ops::function::impls::<impl core::ops::function::Fn<A> for &F>::call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:263:22
   8: proptest::test_runner::runner::call_test::{{closure}}::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:56
   9: <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panic/unwind_safe.rs:274:9
  10: std::panicking::catch_unwind::do_call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:581:40
  11: __rust_try
  12: std::panicking::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:544:19
  13: std::panic::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panic.rs:359:14
  14: proptest::test_runner::runner::call_test::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:16
  15: proptest::test_runner::scoped_panic_hook::internal::with_hook
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/scoped_panic_hook.rs:130:9
  16: proptest::test_runner::runner::call_test
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:256:9
  17: proptest::test_runner::runner::TestRunner::shrink
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:856:30
  18: proptest::test_runner::runner::TestRunner::run_one_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:743:22
  19: proptest::test_runner::runner::TestRunner::gen_and_run_case
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:671:28
  20: proptest::test_runner::runner::TestRunner::run_in_process_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:620:31
  21: proptest::test_runner::runner::TestRunner::run_in_process
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:575:14
  22: proptest::test_runner::runner::TestRunner::run
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:418:18
  23: associative_cache::test_utils::invariants::run_invariant_suite
             at ./src/test_utils/invariants.rs:132:4
  24: invariant_gate::baseline_passes_all_invariants
             at ./tests/invariant_gate.rs:92:2
  25: invariant_gate::baseline_passes_all_invariants::{{closure}}
             at ./tests/invariant_gate.rs:91:36
  26: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
  27: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.

thread 'baseline_passes_all_invariants' (24742) panicked at src/shared/geometry.rs:36:9:
assertion `left == right` failed: total capacity 297 must be divisible by target_num_buckets 2
  left: 1
 right: 0
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:80:14
   2: core::panicking::assert_failed_inner
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:434:23
   3: core::panicking::assert_failed
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:394:5
   4: associative_cache::shared::geometry::BaselineGeometry::for_settings
             at ./src/shared/geometry.rs:36:9
   5: associative_cache::factory::resolve_geometry
             at ./src/factory.rs:24:38
   6: associative_cache::test_utils::invariants::run_invariant_suite::{{closure}}
             at ./src/test_utils/invariants.rs:133:17
   7: core::ops::function::impls::<impl core::ops::function::Fn<A> for &F>::call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:263:22
   8: proptest::test_runner::runner::call_test::{{closure}}::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:56
   9: <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panic/unwind_safe.rs:274:9
  10: std::panicking::catch_unwind::do_call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:581:40
  11: __rust_try
  12: std::panicking::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:544:19
  13: std::panic::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panic.rs:359:14
  14: proptest::test_runner::runner::call_test::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:16
  15: proptest::test_runner::scoped_panic_hook::internal::with_hook
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/scoped_panic_hook.rs:130:9
  16: proptest::test_runner::runner::call_test
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:256:9
  17: proptest::test_runner::runner::TestRunner::shrink
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:856:30
  18: proptest::test_runner::runner::TestRunner::run_one_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:743:22
  19: proptest::test_runner::runner::TestRunner::gen_and_run_case
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:671:28
  20: proptest::test_runner::runner::TestRunner::run_in_process_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:620:31
  21: proptest::test_runner::runner::TestRunner::run_in_process
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:575:14
  22: proptest::test_runner::runner::TestRunner::run
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:418:18
  23: associative_cache::test_utils::invariants::run_invariant_suite
             at ./src/test_utils/invariants.rs:132:4
  24: invariant_gate::baseline_passes_all_invariants
             at ./tests/invariant_gate.rs:92:2
  25: invariant_gate::baseline_passes_all_invariants::{{closure}}
             at ./tests/invariant_gate.rs:91:36
  26: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
  27: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.

thread 'baseline_passes_all_invariants' (24742) panicked at src/shared/geometry.rs:36:9:
assertion `left == right` failed: total capacity 297 must be divisible by target_num_buckets 2
  left: 1
 right: 0
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:80:14
   2: core::panicking::assert_failed_inner
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:434:23
   3: core::panicking::assert_failed
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:394:5
   4: associative_cache::shared::geometry::BaselineGeometry::for_settings
             at ./src/shared/geometry.rs:36:9
   5: associative_cache::factory::resolve_geometry
             at ./src/factory.rs:24:38
   6: associative_cache::test_utils::invariants::run_invariant_suite::{{closure}}
             at ./src/test_utils/invariants.rs:133:17
   7: core::ops::function::impls::<impl core::ops::function::Fn<A> for &F>::call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:263:22
   8: proptest::test_runner::runner::call_test::{{closure}}::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:56
   9: <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panic/unwind_safe.rs:274:9
  10: std::panicking::catch_unwind::do_call
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:581:40
  11: __rust_try
  12: std::panicking::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:544:19
  13: std::panic::catch_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panic.rs:359:14
  14: proptest::test_runner::runner::call_test::{{closure}}
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:258:16
  15: proptest::test_runner::scoped_panic_hook::internal::with_hook
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/scoped_panic_hook.rs:130:9
  16: proptest::test_runner::runner::call_test
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:256:9
  17: proptest::test_runner::runner::TestRunner::shrink
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:856:30
  18: proptest::test_runner::runner::TestRunner::run_one_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:743:22
  19: proptest::test_runner::runner::TestRunner::gen_and_run_case
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:671:28
  20: proptest::test_runner::runner::TestRunner::run_in_process_with_replay
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:620:31
  21: proptest::test_runner::runner::TestRunner::run_in_process
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:575:14
  22: proptest::test_runner::runner::TestRunner::run
             at /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/runner.rs:418:18
  23: associative_cache::test_utils::invariants::run_invariant_suite
             at ./src/test_utils/invariants.rs:132:4
  24: invariant_gate::baseline_passes_all_invariants
             at ./tests/invariant_gate.rs:92:2
  25: invariant_gate::baseline_passes_all_invariants::{{closure}}
             at ./tests/invariant_gate.rs:91:36
  26: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
  27: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.
proptest: FileFailurePersistence::SourceParallel set, but no source file known

thread 'baseline_passes_all_invariants' (24742) panicked at tests/invariant_gate.rs:92:46:
called `Result::unwrap()` on an `Err` value: "P4 capacity_conservation: Test failed: assertion `left == right` failed: total capacity 297 must be divisible by target_num_buckets 2\n  left: 1\n right: 0.\nminimal failing input: AssociativeCacheSettings {\n    num_of_buckets: 9,\n    bucket_size: 33,\n    eviction_policy: ClassicLRU,\n}"
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/panicking.rs:80:14
   2: core::result::unwrap_failed
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/result.rs:1867:5
   3: core::result::Result<T,E>::unwrap
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/result.rs:1233:23
   4: invariant_gate::baseline_passes_all_invariants
             at ./tests/invariant_gate.rs:92:43
   5: invariant_gate::baseline_passes_all_invariants::{{closure}}
             at ./tests/invariant_gate.rs:91:36
   6: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
   7: core::ops::function::FnOnce::call_once
             at /rustc/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.


failures:
    associative_invariants_runs_baseline_suite
    baseline_passes_all_invariants

test result: FAILED. 4 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s

    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/invariant_gate.rs (/tmp/variant-workspace-kt4lv96f/target/debug/deps/invariant_gate-414e19e4143ae900)
error: test failed, to rerun pass `--test invariant_gate`

```
---

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied to a copy of the subject crate at build time (gate-integrity
files excluded).
