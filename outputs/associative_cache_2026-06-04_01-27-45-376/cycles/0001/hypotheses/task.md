# Task: hypotheses-writer (cycle 0001 — cold start)

Cycle: 0001
Number of hypotheses: 3
Output dir: /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_01-27-45-376/cycles/0001/hypotheses

## Framing

This is the first cycle. No prior variants exist. You are proposing the first batch of 3 hypotheses against the permanent research baseline below. Prefer breadth — distinct strategies for bucket-movement — over depth on any single idea.

Every hypothesis in this cycle MUST cite `baseline` as its parent variant. No other parent variant is valid in cycle 0001.

## Authoring scope

Hypotheses must confine their edits to the extension-point impl files below. Propose only ideas expressible within these files:
- src/associative_cache/associative_cache_bucket.rs
- src/shared/eviction_policy.rs

The contract-protected files are writable by no scope: ['src/associative_cache/associative_cache_jni.rs', 'src/factory.rs'].

## Read for context

- /home/yaniv/projects/cache-evolve/inputs/subjects/associative_cache/src — subject Rust source. The code your hypotheses propose to overlay.
- /home/yaniv/projects/cache-evolve/inputs/subjects/associative_cache/manifest.toml — subject manifest (extension points, baseline insight, fitness vector path).
- /home/yaniv/projects/cache-evolve/inputs/subjects/associative_cache/baseline/eval/fitness-vector.json — baseline's measured fitness vector. Numeric anchor your hypotheses target dimensions of.
- /home/yaniv/projects/cache-evolve/inputs/subjects/associative_cache/baseline/insight.md — short prose describing what the baseline does.
