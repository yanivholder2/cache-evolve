---
trace: zipf-small
suite: dev
hidden: false
generator: zipfian
seed: 101
events: 10000
key_universe: 200
signed_off_by: yanivholder2
signed_off_at: 2026-05-17
---

# zipf-small

Zipfian access pattern (α=0.99) over a 200-key universe. Small working set
relative to mid-range cache sizes (e.g. 32×4 = 128 slots), so a cache that
captures the top of the popularity tail will land hit rates well above the
working-set fraction.

## Workload characteristics

- **Skew**: heavy-headed Zipf (α≈1.0). The top ~30 keys account for the
  majority of accesses.
- **Working-set size**: 200 distinct keys; effective hot-set is much smaller.
- **Scan behavior**: none.
- **Temporal locality**: high — same hot keys are reaccessed throughout.

## Why it's in the dev suite

Tests the basic skew-handling story. Any reasonable cache should do well
here; failures suggest a bug in the variant's hot-set retention.
