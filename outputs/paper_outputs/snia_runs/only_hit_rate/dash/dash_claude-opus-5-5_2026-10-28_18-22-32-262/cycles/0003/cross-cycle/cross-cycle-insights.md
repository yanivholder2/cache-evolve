# Cross-Cycle Insights

## Recurring wins

- Stash placement has been the strongest lever. In cycle 1, demoted items going to the stash MRU end won and improved every cell. In cycle 3, promotees landing at the target MRU end won. When a moved item lands at its new tier's eviction end, it is effectively thrown away.
- Protecting reused items from one-hit arrivals keeps helping in tight caches. The two-hit stash exit gate added gains at the small size in both cycles 1 and 2, and survived into cycle 3's winner.
- Each cycle's winner stacked on the one before (MRU demotion, then the gate, then MRU promotion). Making one targeted change per cycle on top of the previous winner has given steady but small gains.

## Recurring failures

- Every extra admission filter (two-hit gate, frequency comparison, LRU-end fresh insertion) has cost hit-rate on `web_0` at the large size. When targets have room, first-hit promotion is already near-optimal. The only variant that recovered this cell was cycle 3's occupancy shortcut.
- Easier routes into the target hurt small caches, as with one-hit return of demoted items and the free-slot shortcut. A one-hit item that takes scarce space starts an eviction cascade.
- Frequency-based admission has been ineffective. Counters stay near zero because moves halve or reset them and promotees enter at the LRU slot. This made the TinyLFU-style comparison behave like plain first-hit promotion.
- Hard LRU-end insertion of fresh puts starves new keys of their probation hit. It has been dropped.
- Counter-reset tweaks on demoted items turned out to be near-neutral. Cycle 3 disproved the cycle 2 explanation that the reset caused the `web_0` regression.
- Per-op-time and metadata-size have never been reported in any cycle. Every cost claim, including the lookup reorder control, is still unverified.

## Open questions

- Does an occupancy-aware gate (skip it when the target has room) stack with MRU-end promotion? Ideally it recovers `web_0` at the large size while keeping the small-size gains.
- What room threshold (more than one free slot, or an occupancy fraction) keeps the shortcut from hurting tight caches?
- Should the gate scale with capacity or occupancy instead of staying fixed at two hits?
- Would delaying a fresh promotee's move to probing (one more target hit) fix the flat `web_0` large-size cell?
- Could frequency admission work if it compared against the bucket's lowest counter and counters survived moves?
- Does target-first lookup order lower get latency? Its hit-rate is identical to baseline, so this needs per-op-time measured.

## Per-policy notes

- The ledger covers one lineage: a bucket-local, two-tier design with a target tier and a small LRU stash. All levers act on the stash and its boundary with the target.
- Cache size splits the results: changes that help in tight caches (gates, stricter admission) tend to hurt in roomy ones, and the reverse. Results split across cells more often than they move together.
- `hm_0` responds to stash-filtering levers at the large size. `web_0` at the large size is the persistent hard cell and favors permissive admission.
