---
trace: zipf-large
suite: dev
hidden: false
generator: zipfian
seed: 102
events: 10000
key_universe: 2000
signed_off_by:
---

# zipf-large

Zipfian (α=0.85) over a 2000-key universe — a much larger working set than
`zipf-small`, with shallower skew. Stresses capacity admission decisions:
the head of the distribution alone won't fit in small caches, and the long
tail is wide enough that random misses are common.

## Workload characteristics

- **Skew**: moderate Zipf (α≈0.85). Hot-set bleeds further into the body of
  the distribution than `zipf-small`.
- **Working-set size**: 2000 distinct keys; effective hot-set ~200.
- **Scan behavior**: none.
- **Temporal locality**: moderate.

## Why it's in the dev suite

Differentiates variants that aggressively admit cold keys (and pollute) from
variants that filter admissions. The contrast against `zipf-small` exposes
size-sensitivity of the admission policy.
