# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0010

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

All three variants built on cycle 0006's v003 (halve on demotion into the stash). Each tried a different lever. v001 added a CLOCK-style decrement to the normal-tier tail survivors on every eviction (ageing at a new boundary). v002 sent ghost hits straight to target Front with counter 1 instead of back into the stash (where credited keys land). v003 added a TinyLFU admission gate on stash→target promotion (aimed at web_0). v002 was the Winner. It gave the largest hm_0 gain the ghost line has produced, plus the first real web_0 gain in the run, both at the large size. The small size was neutral to slightly positive.

## Patterns

- Where a credited key lands mattered more than how much credit it gets or how far the ghost reaches. Ghost reach (c0005) and bonus size (c0006) stalled because credited keys went back into the short-lived stash and were lost again before their next reuse. Skipping the stash turned the credit into target-tier residency. The constraint was structural, and it could not have been found by tuning parameters.
- web_0 finally moved, but through the ghost, not through scan defence. At the large size the ghost catches enough web_0 returners for the bypass to pay off. A gate aimed directly at scans (v003) did nothing: one-hit scan keys never get a stash hit, so they never reach the gate, and target-tail counters sit near 1, so the gate rarely rejects. Whether a cell responds still depends on whether the ghost fires.
- Ageing is safe only when it targets items already judged cold. v003's demotion halving (c0006) did this. v001's decrement on every eviction did not. It hit the Front-landed counter-1 returners inside the tail window, hurt hm_0 most at the small size, and dipped web_0 at both sizes. This is the fifth cycle showing that Front landing and ageing are coupled. Ageing triggered by eviction count also hits small caches hardest.
- Gains from the Winner scale with target-tier capacity. At the small size, counter-1 Front arrivals are evicted before they can be re-hit, and ghost-driven demotion cascades in a contended target can swap out useful residents.
- Per-op-time and metadata-size are still unmeasured, now for seven cycles. The Winner moves ghost-hit puts onto the target-insert and cascade path, and that cost is unknown.

## Open questions

- At the small size, would ghost-admitted keys survive longer in target if they landed slightly deeper than Front, or were briefly exempt from the tail scan?
- Should only some ghost hits bypass the stash, for example only at large sizes or only for keys with a recent ghost entry, so that small-size web_0 avoids harmful cascades?
- Are there other paths where proven-reuse keys are still sent through the stash, such as demoted items with prior target hits, that could also benefit from a direct placement rule?

## Continuation

Make cycle 0007 v002 (ghost hits admitted straight to target Front, counter 1) the new parent. Next, explore how ghost-admitted keys are placed and protected in target, especially to address the small-size capacity limit. Drop eviction-count-driven normal-tier ageing, unless it excludes recently promoted slots. Drop promotion-side admission gates. Insist on per-op-time and metadata-size measurements.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants built on cycle 0007's v002 (ghost hits go straight to target Front with counter 1). Each one targeted the small-size capacity limit from a different side. v001 gave ghost-admitted keys a one-time exemption from the tail-scan victim choice (protecting credited keys). v002 allowed the bypass only when the target tail window had a free or counter-0 slot (gating on contention). v003 sent counter-0 cascade victims to the ghost instead of the stash (where displaced residents go). v002 was the Winner. It gave the largest small-size hm_0 gain, which nobody predicted, and a small web_0 gain at 16384. It gave back part of the parent's large-size web_0 gain.

## Patterns

- The small-size limit comes from who gets displaced, not from how long credited keys survive. Protecting the newcomer (v001) only moved eviction onto proven residents and hurt every cell, the large size most. Refusing the bypass when every candidate is warm (v002) protected those residents and helped. Fast eviction of un-re-hit ghost arrivals is a useful filter, not a bug.
- Where the cascade victim goes does not matter. Sending it to the stash or to the ghost gave identical hit-rates (v003). Stash churn from cascades is not a lever.
- "No cold slot in the window" is a rough admission signal. It separates warm residents from one-time returners well in a contended small target. In a large web_0 target, tail windows can be all warm, so the gate wrongly blocks returners and sends them back to the stash, where they are lost again, as in c0006. The bypass/stash trade-off now depends on cache size and is not settled by any one rule.
- Two variants' insights could not check their overlay diffs, so their mechanisms are inferred. In particular, v001's broad regression might partly come from a reduced counter cap rather than from the exemption itself.
- Per-op-time and metadata-size are still unmeasured, now for eight cycles. v003's only remaining argument was a cost saving, and it cannot be judged.

## Open questions

- Would a relative gate keep v002's small-size protection without blocking valuable large-size returners? One option: bypass when the returner's credit is at least the window's minimum counter, not only when a counter-0 slot exists.
- On a refused bypass, does keeping the ghost entry (letting a second return retry) actually work, or does it just delay the loss?
- Is large-size hm_0 now flat for structural reasons, with no remaining lever in the ghost/bypass family?

## Continuation

Make cycle 0008 v002 (contention-gated ghost bypass) the new parent. Next, refine the gate condition, for example by comparing the returner's credit with the coldest window counter, to win back large-size web_0 while keeping the small-size gains. Drop protecting credited keys inside the victim scan, and drop changes to where cascade victims are sent. Variant insights should confirm the overlay landed as specified, and per-op-time and metadata-size still need to be measured.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

All three variants built on cycle 0008's v002, the contention-gated ghost bypass. Each tried a different way to win back the large-size web_0 loss that v002 caused, without giving up its small-size gains:

- v001 relaxed the gate threshold to admit over counter-1 window minima.
- v002 added the probing neighbour bucket as a second cold-slot placement.
- v003 added a learned per-segment re-hit score as an extra gate branch.

v002 was the Winner, but only because it was the one variant that did no harm. Its gains were noise-level in every cell. In practice the cycle kept the parent and recovered nothing for large-size web_0.

## Patterns

- The single global gate threshold is now used up from both directions. Counter-0-only (c0008) blocks large-size web_0 returners. Counter ≤ 1 (v001) frees them, but it costs more on small-size hm_0, where counter-1 residents are worth protecting. The threshold moves the trade-off between cache sizes. It does not improve both sizes at once.
- More placement options do not help under the same cold-only rule. The neighbour bucket shares the segment and its load, so when the target window is all warm, the neighbour's window is too (v002). The limit is the admission rule, not a lack of nearby slots.
- The learned re-hit signal (v003) did nothing that could be measured. Either the all-warm case rarely triggers it, or "re-hit in target before eviction" is too pessimistic a measure of a returner's value under the demotion cascade. Without counts of how often each gate branch fires, the two cannot be told apart.
- The large-size web_0 gap is real and sits exactly at counter-1 windows. v001 confirmed that all-warm windows at 131072 bottom out mostly at counter 1. The returners worth admitting there compete with counter-1 residents, and on small hm_0 those same counter-1 residents are the ones worth keeping.
- Overlay checks again failed for two of the three variants, so v002's and v003's mechanisms are inferred, not confirmed. Per-op-time and metadata-size are still unmeasured, now for nine cycles. v002's second window read and v003's bitmap both have unknown cost.

## Open questions

- Would a threshold that depends on size or contention keep both sides? For example, ≤ 1 when the target is large or uncontended, and == 0 when it is small or contended, possibly decided from segment occupancy instead of a hard-coded size.
- Does a tie-break that prefers the resident at equal counter keep v001's large-size web_0 gain while removing its small-size hm_0 loss?
- How often does each gate branch actually fire, and how is the v003 score distributed? Instrumentation is needed before any adaptive gate is built on a signal.

## Continuation

Keep cycle 0009 v002 as the parent, or treat 0008 v002 as equivalent, because the gain is noise. Next, try a gate threshold that depends on contention or occupancy: relaxed (≤ 1) only where targets are uncontended, strict (== 0) where they are contended. Separately, try an equal-counter tie-break that favours the resident. Drop placement-spreading (neighbour buckets) and the re-hit-before-eviction feedback score unless they first come with branch-firing instrumentation. Variant insights must confirm the overlay landed, and per-op-time and metadata-size still need to be measured.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
