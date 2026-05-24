# Caffeine simulator invocation

Reference for invoking the Caffeine cache simulator with a Dash-family policy (main `dash` or per-variant `dashvariant`) and consuming its output files.

Consumers:

- `.agents/skills/caffeine-sim/` — single-run skill.
- `.agents/skills/caffeine-sweep/` — multi-trace/multi-size skill.
- `evolve/src/fitness/caffeine_simulator.py` — evolve fitness wrapper (build cdylib + invoke + parse + aggregate).

All consumers go through `scripts/caffeine_runner.py`, which owns invocation. Parsing/aggregation is each consumer's concern.

## Invocation

Caffeine is invoked via `gradlew simulator:simulate -q` against `~/projects/caffeine`. The Rust cdylib is loaded by Caffeine through `-Djava.library.path=<dir-containing-libdash*.so>`.

### Required JVM args

```
-XX:+UseParallelGC
-Xmx8g
-Djava.library.path=<dir of dash.dll | dash_variant.dll>
```

### Required gradle args

```
--maximumSize=<sizes>           # comma-separated; Caffeine wants underscore-thousands (e.g. 16_384)
--title=<label>                 # used in PNG chart title; freeform
-Dcaffeine.simulator.trace.source=files
-Dcaffeine.simulator.files.paths.0=<format>:<trace-file>     # e.g. lirs:multi1.trace.gz
-Dcaffeine.simulator.policies.0=<spec-name>                  # selects the policy implementation
```

The top-level `caffeine.simulator.policies` list defaults to empty — without it Caffeine prints `No active policies in the current configuration` and writes nothing. Per-target spec names:

| Target        | `policies.0` value             |
| ------------- | ------------------------------ |
| `dash`        | `dash.DashPromotionColdRust`   |
| `dashvariant` | `dash.DashVariant`             |

### Target-specific args

Two targets share the same invocation core; only the policy block + class differ.

| Target        | Policy class (Java)        | Config key prefix                | Cdylib filename                          |
| ------------- | -------------------------- | -------------------------------- | ---------------------------------------- |
| `dash`        | `DashRustPolicy`           | `caffeine.simulator.dash.*`      | `dash.dll` / `libdash.so`                |
| `dashvariant` | `DashVariantRustPolicy`    | `caffeine.simulator.dashvariant.*` | `dash_variant.dll` / `libdash_variant.so` |

Geometry must satisfy `numOfSegments × (numOfNormalBuckets + numOfStashBuckets) × bucketSize == maximumSize`, else the policy throws.

Per-target keys to set:

```
# dash:
-Dcaffeine.simulator.dash.numOfNormalBuckets=<n>
-Dcaffeine.simulator.dash.numOfStashBuckets=<n>
-Dcaffeine.simulator.dash.bucketSize=<n>
-Dcaffeine.simulator.dash.policy.0=<name>      # classiclru | timestamplru | lifo | lfu | fifo | swap
-Dcaffeine.simulator.dash.debugMode=<0|1|2>

# dashvariant (same shape, different prefix + only `evictionPolicies` not `policy`):
-Dcaffeine.simulator.dashvariant.numOfNormalBuckets=<n>
-Dcaffeine.simulator.dashvariant.numOfStashBuckets=<n>
-Dcaffeine.simulator.dashvariant.bucketSize=<n>
-Dcaffeine.simulator.dashvariant.evictionPolicies.0=<name>
-Dcaffeine.simulator.dashvariant.debugMode=<0|1|2>
```

## Output files

All paths relative to Caffeine's reports dir (`simulator/build/reports/simulate/` unless overridden via `--outputDir`). Files are **overwritten** between runs — copy them out before re-invoking on a different trace.

| File                       | Format | When                  | Contents                                                                                                                    |
| -------------------------- | ------ | --------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| `hit_rate_<size>.csv`      | CSV    | Always (one per size) | Per-policy metrics row at this size. See column list below.                                                                 |
| `hit_rate.csv`             | CSV    | Multi-size only       | Pivot: rows = policies, columns = `Policy` + one column per cache size. Cells = hit-rate value at that size for that policy. |
| `hit_rate.png`             | PNG    | Multi-size only       | Line chart, 1280×720. X = cache size, Y = hit rate %. One series per policy. Title from `--title`.                          |

Single-size runs skip the combined CSV and chart; Caffeine prints `Did not generate a chart as only one data point` to stdout.

### Per-size CSV columns

`hit_rate_<size>.csv` always includes `Policy` as the first column. Other columns are emitted only if their value is non-zero or the metric is marked required:

```
Policy, Hit Rate, Miss Rate, Hits, Misses, Requests, Evictions,
Admit rate, Requests Weight, Weighted Hit Rate, Weighted Miss Rate,
Adaption, Average Miss Penalty, Average Penalty, Steps, Time
```

One row per policy listed in the config's `policy` / `evictionPolicies` array.

### stdout (not in any file)

- Per-size timing: `<size>: Executed in <duration>`.
- Multi-size end: `Wrote combined report to <path>` + `Wrote chart to <path>`.
- Single-size end: `Wrote report to <path>` + the chart-skipped notice.

### JNI-side logs

The Rust cdylib writes one log file per init, named `<cache_type>_<timestamp>.log`. Default dir is `rust-logs/` relative to the JVM working directory (typically `caffeine/simulator/rust-logs/`). Caffeine itself does not write log files. Log verbosity is controlled by the `debugMode` key per target (`0` = off, `1` = info, `2` = debug).

For `dashvariant`, the log dir can be overridden by setting `dashvariant.logPath`:

```
-Dcaffeine.simulator.dashvariant.logPath=<absolute-or-relative-path>
```

`caffeine_runner.py` exposes this as `--log-path` / `log_path=`. Empty string (or not setting it) keeps the Rust-side default. The `dash` target does not yet support log-dir override.

## Pinning output to a custom directory

Pass `--outputDir=<path>` (Caffeine's `simulator:simulate` flag) to redirect output. Required when multiple consumers share the simulator concurrently or when results must survive past the next run.

## Sweep root layout

Convention shared by `.agents/skills/caffeine-sweep/` and any other multi-trace consumer. One timestamped root per sweep; one subdirectory per trace (named `<trace>_<format>`, e.g. `multi1_lirs`); the per-trace dir holds Caffeine's own output (`hit_rate.csv`, `hit_rate.png`, `hit_rate_<size>.csv`) verbatim. `summary.csv` at the root aggregates the per-trace `hit_rate.csv` files with a prepended `Trace` column.

```
<sweep-root>/<timestamp>/
├── summary.csv          # Trace, Policy, <size1>, <size2>, ...
├── multi1_lirs/
│   ├── hit_rate.csv     # Combined (multi-size): Policy + one column per size
│   ├── hit_rate.png     # Line chart
│   ├── hit_rate_512.csv # Per-size detail (see "Per-size CSV columns" above)
│   ├── hit_rate_1024.csv
│   └── ...
└── <other_traces>/...
```

Each per-trace dir is what Caffeine writes when invoked with `--outputDir=<sweep-root>/<timestamp>/<trace>_<format>` — the runner does not post-process Caffeine's files. Aggregation into `summary.csv` is the sweep consumer's responsibility.
