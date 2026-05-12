---
name: caffeine-sweep
description: Run batch Caffeine simulations across multiple traces and cache sizes. Use when the user wants to sweep, run multiple traces, compare policies, benchmark, or generate hit-rate reports. Triggers on "sweep", "batch run", "run all traces", "compare policies", "hit rate comparison", "benchmark".
---

Run batch Caffeine cache simulations sweeping across traces and cache sizes.

## How it works

`sweep.py` iterates over traces, running the Caffeine simulator for each trace with all cache sizes in one invocation. For each trace it:
1. Builds `dash.dll` via `cargo build --release` (skippable with `--skip-build`)
2. Calls `gradlew simulator:simulate` with the trace, sizes, dash config, and eviction policies passed as system properties
3. Copies Caffeine's generated `hit_rate.csv` and `hit_rate.png` from the reports dir into a per-trace subdirectory before the next run overwrites them
4. After all traces complete, aggregates per-trace CSVs into a single `summary.csv`

## Output files

All output goes to `<output-dir>/<timestamp>/` (default: `caffeine/simulator/build/reports/sweep/<timestamp>/`).

```
<timestamp>/
├── summary.csv                    # All traces combined (Trace, Policy, size1, size2, ...)
├── multi1_lirs/
│   ├── hit_rate.csv               # Combined hit rates for this trace
│   ├── hit_rate.png               # Chart
│   ├── hit_rate_512.csv           # Per-size breakdown
│   ├── hit_rate_1024.csv
│   └── ...
└── multi2_lirs/
    ├── hit_rate.csv
    ├── hit_rate.png
    └── ...
```

## Steps

1. **Resolve traces**: Interpret $ARGUMENTS to select traces from the bundled list below. The user may specify traces by name, format, or description (e.g., "all lirs", "multi1 and sprite", "address traces"). If no traces mentioned, use defaults.
2. Build the `--traces` flag using resolved traces in `format:name.trace.gz` syntax (comma-separated).
3. Run `cargo build --release` (unless `--skip-build` in $ARGUMENTS).
4. Run `python scripts/sweep.py --traces <resolved_traces> <other_flags>`
   Pass through any other flags from $ARGUMENTS (--sizes, --policies, --normal-buckets, --stash-buckets, --bucket-size, --debug, --skip-build, --dry-run, --output-dir).
5. Read and display the summary CSV.

## Default behavior (no args)

Traces: lirs:multi1, lirs:multi2. Sizes: 512..65536 (9 values). Policies: classiclru, lfu, fifo.

## Bundled traces

| Format | Traces |
|--------|--------|
| lirs | 2_pools, backf, cpp, cs, gli, loop, multi1, multi2, multi3, ps, scan, sprite, zigzag |
| address | gcc, gzip, mcf, swim, twolf |
| cache2k | orm-busy, orm-night, web07, web12 |
| scarab | prods, recs, scarab-recs |

Trace format for `--traces`: `<format>:<name>.trace.gz` (e.g., `lirs:multi1.trace.gz`)

## Script args (passed through to sweep.py)

| Arg | Example | Default |
|-----|---------|---------|
| `--traces` | `lirs:multi1.trace.gz,lirs:sprite.trace.gz` | multi1, multi2 |
| `--sizes` | `512,1024,4096` | 512..65536 (9 sizes) |
| `--policies` | `classiclru,lfu` | classiclru,lfu,fifo |
| `--normal-buckets` | `28` | 28 |
| `--stash-buckets` | `4` | 4 |
| `--bucket-size` | `12` | 12 |
| `--debug` | (flag) | off |
| `--skip-build` | (flag) | off |
| `--dry-run` | (flag) | off |
| `--output-dir` | `path/to/dir` | caffeine/.../reports/sweep |

IMPORTANT: Always show the full summary CSV contents when the sweep completes.
