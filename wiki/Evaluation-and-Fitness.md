# Evaluation and Fitness

This page explains how a variant that passes the [gates](Concepts#gates) gets *scored*. It moves from the general (fitness) to the specific (the Caffeine simulator).

## How a variant is scored

Scoring a variant has three steps:

1. **Run the simulator** once for the variant to obtain raw cache metrics.
2. **Run each fitness plugin** that applies to the subject, producing one named dimension each.
3. **Aggregate** the dimensions into the variant's **fitness vector**.

The result is written to the variant's `eval/fitness-vector.json`.

## Fitness plugins

A fitness plugin turns raw metrics into one **dimension** of the fitness vector. Which plugins run is decided by what the subject's manifest declares — there is no dynamic registry, just manifest-gated wiring:

| Plugin          | Runs when                              | Measures                                   |
| --------------- | -------------------------------------- | ------------------------------------------ |
| `hit-rate`      | always                                 | cache hit rate (higher is better)          |
| `per-op-time`   | the subject declares a `[bench]` table | per-operation time from a microbenchmark   |
| `metadata-size` | the subject declares a `[metadata]`    | per-entry metadata overhead in bytes       |

So a minimal subject is scored on hit rate alone; a subject that opts into the extra tables is scored on more dimensions.

### The plugin contract

Every plugin emits a record with four fields:

- `dim` — the dimension name.
- `values` — the per-point detail (free-form list).
- `summary` — a single finite number, the value used for comparison and Pareto ranking.
- `higher_is_better` — the orientation of the dimension.

A record that violates this contract is a hard error. The contract is the seam that lets the framework stay agnostic to *what* a plugin measures.

### The fitness vector

Aggregation produces:

```json
{
  "variant": "<label>",
  "dimensions": [
    { "dim": "hit-rate", "values": [ ... ], "summary": 0.4653, "higher_is_better": true }
  ]
}
```

One `dimensions` entry per plugin. The winner-selection machinery ([Concepts](Concepts#winners-and-the-pareto-front)) consumes the `summary` and `higher_is_better` of each dimension.

### When scoring fails

If the simulator fails (build error, non-zero exit, **timeout**, or missing/malformed output) or a plugin subprocess fails, scoring raises a fitness error. This does **not** crash the cycle: the variant is softly rejected and the rest of the cohort proceeds. The per-variant simulator timeout is configurable (`--simulator-timeout-s`, default 600s).

## Simulators

The simulator is the substrate that actually exercises a cache variant against real access traces and reports metrics. The framework's simulator seam is **subject-agnostic**: a runner knows how to invoke a simulator and build its command line, while every subject-specific detail is supplied declaratively from the subject's manifest `[caffeine]` table. Today the one concrete simulator is **Caffeine**.

```mermaid
flowchart TD
    V["Variant (overlay on the subject crate)"] --> B["Build the native library<br/>cargo build --release → lib&lt;subject&gt;.so"]
    M["Subject manifest.toml<br/>[caffeine], [evaluation]"] --> SIM
    B -->|java.library.path| SIM["Caffeine simulator<br/>(per trace × cache sizes)"]
    SIM -->|JNI: init / get / put / drop| JAVA["The subject's RustPolicy<br/>(Caffeine policy class)"]
    JAVA --> CSV["hit_rate_&lt;size&gt;.csv"]
    CSV --> P["Fitness plugins"]
    M -.->|optional [bench] / [metadata]| P
    P --> FV["fitness-vector.json"]
```

### Caffeine

[Caffeine](https://github.com/ben-manes/caffeine) is a high-performance Java caching library whose simulator can replay access traces against pluggable policies. Evolve runs each variant as a Caffeine *policy* backed by the variant's compiled Rust library, over JNI.

**How the integration works:**

- Each subject's Rust crate compiles to a native shared library (a `cdylib`, e.g. `libdash.so`).
- On the Java side, a small policy class (`<Subject>RustPolicy.java`) loads that library and forwards every cache operation to it.
- The bridge is **pointer-based JNI**: the Rust side returns an opaque pointer when the cache is created, and Java passes that pointer back on every call, finally releasing it when the trace finishes.
- The subject's manifest `[caffeine]` table tells the runner everything it needs — the policy spec name, the config prefix the Java side reads, the cache geometry, and so on — so the runner itself stays subject-agnostic.

**Setting it up (user flow):**

1. Clone the official [`ben-manes/caffeine`](https://github.com/ben-manes/caffeine) as a sibling of this repo, at `../caffeine`.
2. Copy each subject's `<Subject>RustPolicy.java` into its package, and apply the small interim `Registry.java` patch that registers the policies.
3. Build the simulator (`./gradlew simulator:build`).
4. Build the subject libraries (`cargo build --release`) so the `cdylib`s exist on the library path.
5. Let evolve drive the simulator during a run, or invoke it directly for ad-hoc measurements.

> **Note — current vs. planned.** The repo-sibling `../caffeine` layout, the per-subject ownership of the `RustPolicy.java` files, and depending on *official* upstream Caffeine are the target state (tracked in issue #149). Until that lands, the integration points at a local Caffeine fork. This page describes the intended public setup; the [`caffeine-simulator.md`](https://github.com/yanivholder2/cache-evolve/blob/main/docs/caffeine-simulator.md) reference doc and the subject manifests are the live source of truth.

**Going deeper:**

- [`docs/caffeine-simulator.md`](https://github.com/yanivholder2/cache-evolve/blob/main/docs/caffeine-simulator.md) — the full invocation reference (JVM args, gradle args, output files, the overwrite-between-runs caveat).
- Each subject's `manifest.toml` `[caffeine]` table — the exact wiring for that subject.
- [ADR 0005](https://github.com/yanivholder2/cache-evolve/blob/main/docs/adr/0005-single-generic-seam.md) and [ADR 0007](https://github.com/yanivholder2/cache-evolve/blob/main/docs/adr/0007-standalone-subject-crates.md) — the design decisions behind the seam and the per-subject crates.
