---
name: caffeine-sweep
description: Run batch Caffeine simulations across multiple traces and cache sizes. Use when the user wants to sweep, run multiple traces, compare policies, benchmark, or generate hit-rate reports. Triggers on "sweep", "batch run", "run all traces", "compare policies", "hit rate comparison", "benchmark".
---

Multi-trace × multi-size Caffeine sweep. Loops `scripts/caffeine_runner.py simulate` once per trace and aggregates outputs.

Read `docs/caffeine-simulator.md` for the per-invocation reference (JVM args, output files) and the sweep root layout (`## Sweep root layout`).

**Scope: one target per sweep.** Defaults to `--target dash`. To compare `dash` vs `dashvariant`, run the sweep twice with different `OUT_ROOT`s — there is no `--targets` loop. The variant cdylib also only accepts `classiclru` end-to-end today (issue #102), so a variant sweep must restrict `POLICIES=classiclru`.

## Steps

1. **Resolve traces**: Interpret $ARGUMENTS to select traces from the bundled list below. Defaults below if none mentioned.
2. **Resolve cdylib dir**:
   ```bash
   REPO_ROOT=$(git rev-parse --show-toplevel)
   LIB_DIR="$REPO_ROOT/target/release"
   ```
3. **Build** (unless `--skip-build` in $ARGUMENTS): `cargo build --release`.
4. **Choose an output root**:
   ```bash
   OUT_ROOT="${OUTPUT_DIR:-$HOME/projects/caffeine/simulator/build/reports/sweep}/$(date +%Y-%m-%d_%H-%M-%S)"
   mkdir -p "$OUT_ROOT"
   ```
5. **Loop traces**, invoking the runner once per trace with all sizes:
   ```bash
   for trace in "${TRACES[@]}"; do
       name="$(basename "${trace#*:}" .trace.gz)_${trace%%:*}"
       trace_dir="$OUT_ROOT/$name"
       python "$REPO_ROOT/scripts/caffeine_runner.py" simulate \
           --target dash \
           --library-path "$LIB_DIR" \
           --trace "$trace" \
           --sizes "$SIZES" \
           --policies "$POLICIES" \
           --normal-buckets 28 --stash-buckets 4 --bucket-size 12 \
           --output-dir "$trace_dir"
   done
   ```
   `--output-dir` keeps Caffeine from overwriting between traces (per `docs/caffeine-simulator.md`).
6. **Aggregate** the per-trace `hit_rate.csv` files into `$OUT_ROOT/summary.csv` (prepend a `Trace` column). Show the full summary at the end.

## Defaults

- Traces: `lirs:multi1.trace.gz`, `lirs:multi2.trace.gz`.
- Sizes: `512,1024,1536,2048,4096,8192,16384,32768,65536`.
- Policies: `classiclru,lfu,fifo`.

## Bundled traces

| Format  | Traces                                                                 |
|---------|------------------------------------------------------------------------|
| lirs    | 2_pools, backf, cpp, cs, gli, loop, multi1, multi2, multi3, ps, scan, sprite, zigzag |
| address | gcc, gzip, mcf, swim, twolf                                            |
| cache2k | orm-busy, orm-night, web07, web12                                      |
| scarab  | prods, recs, scarab-recs                                               |

Trace selector format: `<format>:<name>.trace.gz` (e.g. `lirs:multi1.trace.gz`).
