---
trace: scan-burst
suite: holdout
hidden: true
generator: scan-burst
seed: 202
events: 10000
hot_universe: 120
scan_universe: 600
burst_length: 1200
burst_every: 800
signed_off_by:
---

# scan-burst (HELD-OUT)

**HIDDEN FROM CONTEXT-BUNDLE ASSEMBLY.** Do not surface this profile or its
trace contents to the variant-implementer or hypothesis-proposer subagents.

A more adversarial cousin of dev `scan-mixed`: every 800 events triggers a
*long* (~1200-access) sequential scan over 600 cold keys, repeated until the
quota is satisfied. The scan is large enough to evict the entire hot set on
caches that don't actively protect it.

## Workload characteristics

- **Skew**: hot-set is uniform; scans are sequential and longer than the
  cache itself in the small-size sweeps.
- **Working-set size**: 120 hot + 600 scan = 720 distinct keys.
- **Scan behavior**: very heavy. Roughly half of all events are scan accesses.
- **Temporal locality**: very high on hot set when not in a burst.
