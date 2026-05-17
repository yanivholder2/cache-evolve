---
trace: uniform
suite: holdout
hidden: true
generator: uniform
seed: 203
events: 10000
key_universe: 1500
signed_off_by:
---

# uniform (HELD-OUT)

**HIDDEN FROM CONTEXT-BUNDLE ASSEMBLY.** Do not surface this profile or its
trace contents to the variant-implementer or hypothesis-proposer subagents.

Uniform random over 1500 keys. There is no skew to exploit; hit rate is
bounded by `cache_capacity / key_universe`. Any variant that claims a
hit-rate improvement here is either wrong or has a bug.

## Workload characteristics

- **Skew**: none.
- **Working-set size**: 1500 keys.
- **Scan behavior**: none.
- **Temporal locality**: none.

## Why it's in the held-out suite

Sanity check / regression sentinel. Variants that overfit to skew-bearing
dev traces shouldn't catastrophically regress here. A significant drop
relative to LRU on this trace is a strong signal that the variant is
mis-classifying recency.
