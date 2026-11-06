# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0009

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants built on cycle 0005's v001 (a direct-mapped ghost sized to the cache, admitting at counter 1). Each one isolated a different explanation for why large-size hm_0 stays stuck. v001 raised the ghost-hit credit to 2 (bonus strength). v002 doubled the ghost table (reach). v003 halved an item's counter when it was demoted into the stash (where ageing happens). v003 was the Winner. It was the only variant with a small, consistent hm_0 gain at both sizes, including the first movement at the large size, and web_0 was flat to noise-level negative.

## Patterns

- Ghost reach and ghost bonus strength are now both used up. A credit of 2 barely moved small-size hm_0, because a credit of 1 already beats the mostly counter-0 stash tail. A 2× table did nothing at the large size and slightly hurt the small size, because older returners diluted the bonus. Neither the bonus nor reach is what limits large-size hm_0.
- What helped was changing who the ghost-credited arrivals compete against, not making the credit itself bigger. Taking away the stale target-tier counts that demoted items carried into the stash helped. This is the fourth cycle in which counter ageing turned out to be the active lever. It is also the first time ageing was added at a point where no hit is penalised, and that is why it was safe, unlike removing the stash-hit halving.
- Gains still depend on the ghost firing. That is why hm_0 responds and web_0 does not. web_0's only reactions this cycle were noise-level dips: demoted items returning to target with less protection (v003), and over-credited scan keys (v001).
- The effects are getting smaller each cycle. Every lever in this family (window, ageing, ghost reach, bonus, ageing point) now yields small gains, which suggests the current structure is close to its limit.
- Per-op-time and metadata-size are still not in the fitness vector, now for six cycles. v002's extra metadata was reported only as an estimate.

## Open questions

- Do other tier boundaries also benefit from moving ageing there? For example: halving on stash→target promotion, or ageing applied inside the normal tier's tail scan.
- Does large-size hm_0 lose reuse through exit paths that never reach the ghost (keys leaving other than by stash drops)?
- Would a stronger demotion decay (a quarter, or a reset) help hm_0 more, or start hurting web_0 at the large size?
- Is there any lever at all for web_0, perhaps scan detection or admission filtering on the normal tier rather than the stash?

## Continuation

Make cycle 0006 v003 (halve on demotion into the stash) the new parent. Drop further ghost-size and ghost-bonus tuning; both are ruled out. Next, test where ageing is applied at other boundaries (promotion back into target, normal-tier eviction), and check whether hm_0's lost keys leave through paths the ghost never records. Given the shrinking gains, consider one structurally different hypothesis aimed at web_0, and push again for per-op-time and metadata-size to be measured.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
