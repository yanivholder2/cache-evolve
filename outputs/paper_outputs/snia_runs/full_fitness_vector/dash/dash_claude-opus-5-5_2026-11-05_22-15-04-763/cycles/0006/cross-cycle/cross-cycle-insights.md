# Cross-cycle insights (cycles 0001–0006)

## Recurring wins

- **Keeping proven items resident raises hit rate every time.** The two-hit gate (0001), MRU demotion to stash, both gated (0002) and ungated (0003), probing overflow (0002, 0003) and the probing room gate (0004) each gained hit rate on every cell tested, or close to it. Every one of them keeps re-referenced items alive longer.
- **hm_0 at the large size is the cell that shows retention effects.** In every cycle, changes that help retention gain most there, and changes that hurt retention lose most there. Use it as the first place to check any retention change.
- **Probing overflow cut the cascades that `put` p99 pays for, while still letting items leave stash.** It is the only cascade reduction that improved tails without moving the cost to another path (0003).
- **Small, cheap per-item state is affordable.** Reusing `lfu_counter` added no metadata.
- **Current best line:** gate + MRU demotion + probing overflow + probing room gate (0004 v002). The variant with a single bounded deferral (0005 v002) is a branch of it: better `put` p99, worse `get` tail and small-cache hit rate. Neither has a Winner yet. No variant matches baseline `get` p50.

## Recurring failures

- **Skipping or deferring cascades only moves their cost.** Unbounded deferral (0004) kept items in stash permanently and made `put` p99 much worse. Single deferral (0005) and the ungated overflow branch (0004) moved the cost to `get`. A change that skips promotions only works if items keep flowing out of stash.
- **The `get` p50 gap has not been found on the common hit path.** Lookup reordering with stash last (0003, worse because stash hits are a large share of hits), scan direction (0005), removing the first-hit MRU move (0002) and skipping the near-MRU reorder (0006) all left p50 flat or worse.
- **Weak signals for the gate.** A liveness signal from a single normal-tier stay (0005) lost hit rate everywhere. A gate on how full the stash bucket is (0006) never fires, because stash is nearly always full at steady state.
- **The ungated MRU-demotion line is not cheaper.** It reaches the same slower `get` band as the gated line (0003), and adding overflow to it costs more `get` p99 than it saves on `put` (0004). Drop this line.
- **Small p99 changes are unreliable.** Cost-only variants with identical policy made a p99 worse on paths they don't touch (0005, 0006). The cause is noise or codegen/inlining side effects. Don't decide anything from a small tail change without a rerun.
- **Build infrastructure:** Maven/Gradle 429s killed variants in 0001. Treat build failures as untested ideas, not as negative results.

## Open questions

- **Stash probe cost:** the only remaining suspect for `get` p50 is the stash probe on every normal-tier hit. Possible fixes are a cheaper stash membership check or a skip filter for keys known not to be in stash. Moving stash to the end of the lookup has already been tried and was worse.
- **Fused stash-promotion rotate (0006 v003):** it improved `get` p99 but made `put` p99 worse. Does keeping it to the third-hit path only, or marking it non-inline, keep the `get` gain without the `put` cost?
- **Better deferral signal than fill level:** for example, whether the stash slot that would be evicted holds an item still waiting for its second hit. The goal is to keep 0005 v002's `put` p99 gain without its small-cache hit-rate loss.
- **Size-dependent trade-off:** longer stash residency helps the large cache and hurts the small one. Could the policy adapt to cache size?
- **Is `get` p50 parity with baseline achievable at all** while stash hits stay a large share of hits, or is part of the gap simply the price of the hit-rate gains?

## Per-policy notes

- **Dash (the only policy studied so far):**
  - The stash-hit promotion chain (remove → promote → demote cascade) drives `get` latency. It scales with how many stash hits there are, not with the gate itself.
  - `put` p99 follows stash pressure and how many cascades `put` has to absorb.
  - Items placed in probing pay a lookup penalty: they are found last and can be evicted by a neighbour's promotion.
  - MRU demotion for every demoted item is settled policy. Keep the two-hit gate.
