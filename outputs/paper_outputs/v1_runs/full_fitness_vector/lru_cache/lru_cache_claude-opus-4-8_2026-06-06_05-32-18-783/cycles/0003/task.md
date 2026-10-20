# Task: cycle-insight-writer

Cycle: 0003
Output dir: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0003

## Winners

- v003

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0003/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0003/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0003/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0003/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0003/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants forked from baseline, each a single-axis lever:
segmented-LRU for hit-rate (v001), narrow `u32` sentinel links for
metadata-size (v002), and a fast inline integer hasher for per-op-time
(v003). The hasher swap won — the only change that improved a hot-path
dimension without paying a hard regression elsewhere.

## Patterns

- Per-op-time was the contested axis: both losing variants moved their
  target dimension but bought it with added per-op latency (extra
  promote/demote branching in v001, per-access index-width casts in v002).
- The Winner improved per-op-time directly and even picked up a free
  metadata side-win (dropping the default hasher's seed) — cheap,
  near-pure changes on the hot path beat changes that trade one axis
  against another.
- Representation/policy changes that widen the node or sprinkle
  conversions did not pay for themselves in locality.

## Open questions

- The SLRU hit-rate gain (v001) is real and targets the LIRS workload's
  weakness — can it be made cheaper so it stops trading against latency?
- v002 flagged that narrow links could be kept without per-access casts;
  is there a cast-free encoding that lands the metadata win?
- v003's median win came with a get tail-latency regression — is the
  hasher change strictly worth it, or does the tail need attention?

## Continuation

Build on v003 as the per-op-time base. Pursue the narrow-link
metadata win again but without per-access width casts, and probe whether
the SLRU hit-rate lever can be reintroduced without its latency cost.
Drop the naive `u32`-cast-everywhere approach as tried-and-regressed.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cycle revisited the two open levers from Cycle 0001 and probed a
hot-path skip. The standout Winner combined the SLRU hit-rate policy
with the integer hasher and segment-aware fast paths, defending the
record hit-rate while pulling latency back to hot-path-class — proving
the policy and latency levers are orthogonal and stackable. The
narrow-`u32`-link metadata re-encoding also landed as a Winner, hitting
the smaller node, while a standalone MRU-guard branch lost.

## Patterns

- Hit-rate (segmentation policy) and per-op-time (cheap hasher + skipping
  no-op list surgery) are independent axes: the cycle's best Variant
  optimized both at once with neither paying for the other.
- Skipping redundant list surgery only pays when the skip condition is
  cheaply predictable. Conditioned on segment position (already-MRU within
  protected) it helped; as a bare data-dependent `tail == idx` branch on
  the universal hot path it mispredicted and regressed both median and
  tail — the same idea won in one framing and lost in another.
- The narrow-link metadata win is genuinely achievable and centralising
  the casts keeps the slowdown modest, but re-encoding `Option` niches as
  explicit sentinels still costs measurable median and tail latency — a
  "pure representation" change is not actually latency-free.

## Open questions

- Can the metadata (24-byte node) and the latency-optimal hot path coexist
  in one Variant, or does any sentinel encoding inherently tax per-op-time?
- The winning fast path skipped surgery using a predictable position
  check — are there other no-op-touch cases (same-segment, head) skippable
  the same cheap way without inviting a mispredicting branch?

## Continuation

Carry forward the combined SLRU-plus-fast-hasher Variant as the new base —
it holds both the hit-rate record and competitive latency. Next cycle
should try folding the narrow-`u32`-link metadata win onto that base and
measure whether the encoding's latency tax survives once the fast paths
absorb it. Drop bare data-dependent MRU-guard branches on the universal
hot path; only pursue surgery-skips gated on cheaply predictable position
conditions.


## Rejected variants

- v001 — implement-and-gate
  [('/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/baseline/eval/fitness/hit-rate.json', '/tmp/variant-workspace-hpc150xj/baseline/eval/fitness/hit-rate.json', '[Errno 122] Disk quota exceeded'), ('/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/baseline/eval/fitness/metadata-size.json', '/tmp/variant-workspace-hpc150xj/baseline/eval/fitness/metadata-size.json', '[Errno 122] Disk quota exceeded'), ('/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/baseline/eval/fitness/per-op-time.json', '/tmp/variant-workspace-hpc150xj/baseline/eval/fitness/per-op-time.json', '[Errno 122] Disk quota exceeded'), ('/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/baseline/eval/caffeine/gradle.stdout.log', '/tmp/variant-workspace-hpc150xj/baseline/eval/caffeine/gradle.stdout.log', '[Errno 122] Disk quota exceeded'), ('/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/baseline/eval/caffeine/gradle.stderr.log', '/tmp/variant-workspace-hpc150xj/baseline/eval/caffeine/gradle.stderr.log', '[Errno 122] Disk quota exceeded'), ('/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/baseline/eval/caffeine/hit_rate_2048.csv', '/tmp/variant-workspace-hpc150xj/baseline/eval/caffeine/hit_rate_2048.csv', '[Errno 122] Disk quota exceeded'), ('/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/baseline/eval/fitness-vector.json', '/tmp/variant-workspace-hpc150xj/baseline/eval/fitness-vector.json', '[Errno 122] Disk quota exceeded'), ('/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/baseline/insight.md', '/tmp/variant-workspace-hpc150xj/baseline/insight.md', '[Errno 122] Disk quota exceeded'), ('/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src/lib.rs', '/tmp/variant-workspace-hpc150xj/src/lib.rs', '[Errno 122] Disk quota exceeded'), ('/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src/logging.rs', '/tmp/variant-workspace-hpc150xj/src/logging.rs', '[Errno 122] Disk quota exceeded'), ('/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src/lru_cache/lru_cache_jni.rs', '/tmp/variant-workspace-hpc150xj/src/lru_cache/lru_cache_jni.rs', '[Errno 122] Disk quota exceeded'), ('/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src/lru_cache/mod.rs', '/tmp/variant-workspace-hpc150xj/src/lru_cache/mod.rs', '[Errno 122] Disk quota exceeded'), ('/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src/test_utils/mod.rs', '/tmp/variant-workspace-hpc150xj/src/test_utils/mod.rs', '[Errno 122] Disk quota exceeded'), ('/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src/test_utils/invariants.rs', '/tmp/variant-workspace-hpc150xj/src/test_utils/invariants.rs', '[Errno 122] Disk quota exceeded'), ('/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src/bin/evolve_metadata_size.rs', '/tmp/variant-workspace-hpc150xj/src/bin/evolve_metadata_size.rs', '[Errno 122] Disk quota exceeded'), ('/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src/bin/evolve_per_op_time.rs', '/tmp/variant-workspace-hpc150xj/src/bin/evolve_per_op_time.rs', '[Errno 122] Disk quota exceeded'), ('/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/tests/contract_gate.rs', '/tmp/variant-workspace-hpc150xj/tests/contract_gate.rs', '[Errno 122] Disk quota exceeded'), ('/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/tests/invariant_gate.rs', '/tmp/variant-workspace-hpc150xj/tests/invariant_gate.rs', '[Errno 122] Disk quota exceeded'), ('/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/tests/lru_cache_integration.rs', '/tmp/variant-workspace-hpc150xj/tests/lru_cache_integration.rs', '[Errno 122] Disk quota exceeded'), ('/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/tests/universal_invariant_gate.rs', '/tmp/variant-workspace-hpc150xj/tests/universal_invariant_gate.rs', '[Errno 122] Disk quota exceeded'), ('/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/Cargo.toml', '/tmp/variant-workspace-hpc150xj/Cargo.toml', '[Errno 122] Disk quota exceeded'), ('/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/manifest.toml', '/tmp/variant-workspace-hpc150xj/manifest.toml', '[Errno 122] Disk quota exceeded')]

## Write

Exactly one file: `cycle-insight.md` in the output dir.
