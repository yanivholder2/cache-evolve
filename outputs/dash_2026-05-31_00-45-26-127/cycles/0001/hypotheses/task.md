# Task: hypotheses-writer (cycle 0001 — cold start)

Cycle: 0001
Number of hypotheses: 3
Output dir: /home/yaniv/projects/cache-evolve/outputs/dash_2026-05-31_00-45-26-127/cycles/0001/hypotheses

## Framing

This is the first cycle. No prior variants exist. You are proposing the first batch of 3 hypotheses against the permanent research baseline below. Prefer breadth — distinct strategies for bucket-movement — over depth on any single idea.

Every hypothesis in this cycle MUST cite `baseline` as its parent variant. No other parent variant is valid in cycle 0001.

## Authoring scope

Hypotheses must confine their edits to the extension-point impl files below. Propose only ideas expressible within these files:
- src/dash/dash_bucket.rs
- src/shared/eviction_policy.rs
- src/shared/item.rs

The contract-protected files are writable by no scope: ['src/dash/jni.rs', 'src/factory.rs'].

## Read for context

- /home/yaniv/projects/cache-evolve/inputs/subjects/dash/src — subject Rust source. The code your hypotheses propose to overlay.
- /home/yaniv/projects/cache-evolve/inputs/subjects/dash/manifest.toml — subject manifest (extension points, baseline insight, fitness vector path).
- /home/yaniv/projects/cache-evolve/inputs/subjects/dash/baseline/eval/fitness-vector.json — baseline's measured fitness vector. Numeric anchor your hypotheses target dimensions of.
- /home/yaniv/projects/cache-evolve/inputs/subjects/dash/baseline/insight.md — short prose describing what the baseline does.

## Scope correction

The previous attempt named files outside the writable surface. Rewrite every hypothesis so its `## Changes` edits only these extension-point impl files:
- src/dash/dash_bucket.rs
- src/shared/eviction_policy.rs
- src/shared/item.rs

Out-of-scope files named last attempt:
- H01.md: ['factory.rs']
- H02.md: ['factory.rs']

Express the same idea through one of the in-scope files instead.

## Scope correction

The previous attempt named files outside the writable surface. Rewrite every hypothesis so its `## Changes` edits only these extension-point impl files:
- src/dash/dash_bucket.rs
- src/shared/eviction_policy.rs
- src/shared/item.rs

Out-of-scope files named last attempt:
- H01.md: ['factory.rs', 'jni.rs']

Express the same idea through one of the in-scope files instead.
