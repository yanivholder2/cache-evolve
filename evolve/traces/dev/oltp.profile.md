---
trace: oltp
suite: dev
hidden: false
generator: oltp
seed: 104
events: 10000
hot_universe: 150
cold_universe: 2000
hot_prob: 0.92
signed_off_by:
---

# oltp

OLTP-class workload: 92% of accesses target a small hot set of 150 keys
(index pages, row caches), 8% target a large cold tail of 2000 keys
(occasional one-shot lookups). Mirrors the working-set shape of an OLTP
database buffer pool.

## Workload characteristics

- **Skew**: bimodal — uniform within hot set, uniform within cold tail.
- **Working-set size**: hot ~150; cold 2000 one-shots.
- **Scan behavior**: none, but cold tail acts as constant low-grade pollution.
- **Temporal locality**: very high on hot set; effectively none on cold tail.

## Why it's in the dev suite

The cold one-shots are the canonical case where naive LRU is wasteful: each
miss inserts a key that will not be reused. Variants with admission
filtering (e.g. TinyLFU-style) should clearly separate from plain LRU here.
