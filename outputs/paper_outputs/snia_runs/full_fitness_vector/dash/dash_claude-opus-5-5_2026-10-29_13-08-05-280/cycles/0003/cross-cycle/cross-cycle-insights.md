## Recurring wins

- Admission gating based on reuse evidence has improved hit-rate in every cycle. The stash reuse gate from 0001 became the lean threshold-only gate in 0002, which became the full-target occupancy guard in 0003. The gains come from deciding *who enters* a tier, not *where* an item lands or *how quickly it returns*.
- Fewer multi-bucket moves (promotions, demotion cascades, forced demotions into stash) means better hit-rate and better put/get latency together. Every clear mechanistic Winner reduced stash churn, and every loser added to it.
- Keeping the common target-hit path free of bookkeeping pays off. Removing per-hit counter writes recovered median get. The cheapest form of each guard (a threshold instead of a counter plus a comparison) kept most of the hit-rate gain.
- Large cache sizes, and hm_0 especially, are where reuse-aware changes win or lose hit-rate. Small-cache cells barely respond or move the opposite way.

## Recurring failures

- Placement based only on free space or capacity (balanced or free-slot placement, even when limited by a reuse flag) never moved hit-rate under pressure. It is dropped.
- Extra "protect reused items" mechanisms stacked on the gate add little. MRU demotion landing and reuse flags mostly duplicate what the gate already captures.
- Anything that makes coming back cheaper (re-admission credits, faster re-promotion) increases churn between reused residents and evicts fresh items still waiting for their second hit. It lowered hit-rate in every cell.
- Latency-only "wins" have been unreliable for three cycles straight. Variants whose diffs could not explain their latency changes still moved a lot, and none of them has been checked against a re-run of the baseline.
- Narrowing `lfu_counter` alone does not shrink Item. `timestamp: Instant` sets the size, and any field of 8 bytes or less disappears into its alignment padding.

## Open questions

- How much noise does per-op-time carry? Re-run an identical baseline before trusting any more latency-only differences. This has been raised every cycle and is still not done.
- Should gate thresholds scale with cache or stash size? A fixed threshold helps large caches and slightly hurts small ones, because a key can't collect enough hits during a short stash stay.
- Can a write-free or size-gated victim guard win back the large-cache hits that were lost when the victim comparison was removed?
- Can `timestamp` become a tick of 4 bytes or less, or be dropped for policies that never read it, so that Item shrinks (to about 24 bytes) without hurting hit-rate?
- Does better victim choice inside a full target add anything beyond the door guard?
- What limits web_0 at the large size, now that placement tweaks have been ruled out?

## Per-policy notes

- Dash stash → target admission: this is the productive line. The current baseline is a threshold-only reuse gate plus a stricter threshold for entering a *full* target. Hit-rate gains come from here.
- Dash target → probing promotion (hottest-in-target rule): insert position and promotion are coupled, because landing position sets how quickly an item climbs on its next hit. Changes here easily add churn. Requiring reuse before promotion is still untested.
- Dash demotion/landing (MRU, Back positions): gives a small, inconsistent benefit once the gate exists. Low priority.
- Item layout / metadata-size: unchanged after three cycles. The `u8` counter is in place as a prerequisite, and the next lever is `timestamp`.
