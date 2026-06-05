# Caffeine simulator invocation

Reference for invoking the Caffeine cache simulator with a JNI-backed (native cdylib) policy and consuming its output files.

## Invocation

Caffeine is invoked via `gradlew simulator:simulate -q` from a Caffeine checkout. The checkout location is resolved by `scripts/caffeine_runner.py`: the `CAFFEINE_DIR` env var if set, otherwise a `caffeine/` sibling of the cache-evolve repo root (so `~/projects/cache-evolve` pairs with `~/projects/caffeine`). A native policy is loaded by Caffeine through `-Djava.library.path=<dir-containing-the-cdylib>`.

### Required JVM args

```
-XX:+UseParallelGC
-Xmx8g
-Djava.library.path=<dir of the cdylib: *.dll | lib*.so | lib*.dylib>
```

### Required gradle args

```
--maximumSize=<sizes>           # comma-separated; Caffeine wants underscore-thousands (e.g. 16_384)
--title=<label>                 # used in PNG chart title; freeform
-Dcaffeine.simulator.trace.source=files
-Dcaffeine.simulator.files.paths.0=<format>:<trace-file>     # e.g. lirs:multi1.trace.gz
-Dcaffeine.simulator.policies.0=<spec-name>                  # selects the policy implementation
```

The top-level `caffeine.simulator.policies` list defaults to empty — without it Caffeine prints `No active policies in the current configuration` and writes nothing.

### Policy selection and config

`-Dcaffeine.simulator.policies.0=<spec-name>` chooses which policy implementation runs. Each policy reads its own settings under a config-key prefix (`caffeine.simulator.<prefix>.*`); the available keys are defined by the policy class itself. A JNI-backed policy additionally requires the matching cdylib on `java.library.path`. Consult the policy's source for the exact keys and any geometry/validity constraints it enforces.

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

One row per policy listed in the config's `policy` array.

### stdout (not in any file)

- Per-size timing: `<size>: Executed in <duration>`.
- Multi-size end: `Wrote combined report to <path>` + `Wrote chart to <path>`.
- Single-size end: `Wrote report to <path>` + the chart-skipped notice.

### JNI-side logs

A native cdylib policy may write its own log file(s), separate from anything Caffeine writes (Caffeine itself does not write log files). Conventionally the log directory and verbosity are controlled by the policy's own config keys (e.g. a `logPath` key and a `debugMode` key). Consult the policy's source for the exact keys and defaults.

## Pinning output to a custom directory

Pass `--outputDir=<path>` (Caffeine's `simulator:simulate` flag) to redirect output. Required when multiple runs share the simulator concurrently or when results must survive past the next run.
