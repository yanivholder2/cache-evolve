## Recurring wins

- No cross-cycle evidence yet: only one cycle has finished. The leading candidate is the two-hit stash promotion gate (v002). It raised hit rate on every trace and size tested, with the biggest gain at the larger size, on hm_0 most of all. A later cycle needs to reproduce it before it counts as a recurring win.
- Storing per-item state in unused metadata fields (e.g. `lfu_counter`) kept metadata size unchanged. Worth reusing.

## Recurring failures

- Infrastructure: the simulator build hit HTTP 429 rate limits from Maven/Gradle, and two of three variants (v001, v003) failed before any evaluation. Treat these as build failures, not evidence against the policy ideas.
- Hit-rate gains that add movement within the stash (an extra MRU move on the first hit) slowed `get` at both p50 and p99 and also slowed `put`. Only one cycle shows this so far, so it is not yet a confirmed trend.

## Open questions

- Is v002's `put` slowdown real (stash buckets filling and evicting more often while items wait for promotion) or benchmark noise?
- Does the gate keep its hit-rate gain if the first stash hit only sets the counter and skips the MRU move?
- H01 (demote to the stash MRU end) and H03 (balanced target/probing-bucket promotion) were never measured. Do they help alone, and do they stack with the gate? H03 might offset the gate's latency cost.

## Per-policy notes

- Dash: the working theory is that weakly re-referenced items flush the small normal tier. Gating admission to the protected tier supports this theory, but costs per-op latency. Keep the gate as the baseline, and do not trade its hit-rate gain away for latency.
