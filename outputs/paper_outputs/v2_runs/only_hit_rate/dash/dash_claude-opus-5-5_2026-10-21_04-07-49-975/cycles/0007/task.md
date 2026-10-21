# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0007

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants were built on cycle-0003 v002 (second-chance stash scan) and each tried a lever not tested before. v001 added non-resident ghost history with a pre-credit. v002 added a TinyLFU-style frequency duel at the stash→target edge. v003 changed where fresh keys land in the stash, using bimodal (BIP) insertion. Only v003 moved hit-rate; v001 and v002 were flat within noise. This breaks the plateau declared in cycle 0003.

## Patterns

- Insert position of *fresh* stash keys is a real lever, unlike normal-tier landing (0001) and stash victim choice (0003). It decides which item the scan reaches first. Placing fresh keys at the LRU end splits each stash bucket into a probation slot that one-touch keys churn through and a protected victim cache for ex-residents. The scan also stops aging credited items.
- Changing the promotion counter does nothing for keys that never get a hit while resident. Both nulls failed for this reason. Ghost pre-credit only lowers a bar that keys with long reuse distances never reach, and the per-bucket ghost window was far too short to cover them anyway.
- Comparative admission has nothing to work with while Front landing makes the normal-tier eviction candidate almost always a fresh arrival with counter 0. In this design, normal-tier frequency and position are coupled, so any frequency check reads what position already tells it.
- The winning pattern is still "fresh keys face a harsh bar, proven ex-residents are protected", the same SLRU/2Q split as in 0002. This cycle applied it with insert position instead of thresholds.
- This cycle ignored the 0003 continuation. It was hit-rate only, on the same single LIRS trace and cache size, with no per-op-time or metadata-size measurements.

## Open questions

- BIP's cost is that keys with moderate reuse distance must hit before the next insert into their bucket. Would a tuned BIP period or a probation area wider than one slot do better?
- Would a ghost window covering roughly cache-size evictions help if a ghost hit changed placement or residency (e.g. admitting straight to target or MRU) instead of the counter?
- Does BIP's scan resistance hold on other traces and sizes, or does it overfit LIRS loops?
- Are per-op-time and metadata-size still unmeasured for every winner, including 0003 v003's cascade-free routing?

## Continuation

Build on v003 (BIP fresh-key insertion). Probe its parameters, such as the BIP period and the probation depth. If ghosts are revisited, they need a much larger window and a ghost hit must change placement, not the counter. Drop counter-only pre-credit and frequency duels against the LRU-end resident, unless victim selection becomes frequency-based. Next cycle should finally measure per-op-time and metadata-size and run at least one more trace or cache size.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants were built on cycle-0004 v003 (BIP fresh-key insertion), and all three tried to rescue the fresh keys with moderate or long reuse distance that BIP sacrifices. v001 widened probation to two slots. v002 made the insertion choice adaptive with set-dueling DIP. v003 added a ghost table about one cache in size, where a ghost hit sends the returning key to the stash MRU end. Only v003 gained, and only a little. v001 was flat to slightly worse and v002 lost hit-rate.

## Patterns

- What decides the outcome is whether a variant takes protected capacity away from ex-residents. v001 handed protected slots to probation, and v002 restored MRU insertion in some buckets and phases. Both weakened the victim cache for pre-credited ex-residents, and both lost or broke even. v003 left that region alone and promoted only keys with evidence that they return, and it was the one that won.
- On this trace a protected slot holding an ex-resident is worth at least as much as an extra chance for a fresh key. The stash split is close to zero-sum, so a fresh key should get past probation only on a selective, evidence-based signal, not a blanket one.
- Cycle 0004 concluded that insert position is the lever and counter credit is not. This cycle confirms it a second time: ghosts failed when they changed the counter (0004) and worked once they changed placement and covered a window about the size of the cache (0005).
- BIP's gain does not appear to depend on loop phases on this trace. Adaptive fallback to MRU cost more than it saved. However, the duel's miss signal was noisy and its leader groups were small, so this result does not fully rule DIP out.
- The gains are getting small. The ghost win suggests that most of the remaining misses are true one-touch keys or keys reused at distances beyond one cache's worth of evictions.
- For the third cycle running, no variant writer could inspect the overlay source, and every measurement again came from a single LIRS trace and a single size, covering hit-rate only. The metadata the ghost table adds is not captured by any fitness dimension.

## Open questions

- Would a larger ghost window, or a ghost hit that also protects the key from the next scan pass, recover more of the long-reuse misses?
- Is the ghost table's metadata and per-op cost justified by its small hit-rate gain? Neither has been measured.
- Is tuning `BIP_PERIOD` worth trying, given that it is the only fresh-key survival knob that takes no capacity from the protected region?
- Do the BIP and ghost gains hold on other traces and cache sizes?

## Continuation

Build on v003 (ghost-guided MRU placement). Probe the ghost window size and whether a ghost hit should also give scan protection, and optionally tune `BIP_PERIOD`. Drop wider probation and set-dueling DIP, along with any other change that takes capacity from the protected ex-resident region. Gains on this single trace are now small. Next cycle should put measuring metadata-size and per-op-time first (the ghost table makes this urgent) and evaluate on at least one more trace or cache size.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants were built on cycle-0005 v003 (ghost-guided MRU placement), and each probed one of the levers that cycle flagged. v001 made the ghost window 4× larger. v002 gave ghost-admitted keys a one-shot pass through the second-chance scan. v003 made BIP's blind MRU admission 4× rarer. v003 won with a small gain. v002 was barely positive, close to noise, and v001 lost hit-rate.

## Patterns

- The zero-sum rule for protected stash capacity held a third time, now from the other side. The winner did not add protection for anyone. It took blind admissions out of the protected region, and evidence-based residents filled the space. The loser let weaker evidence in: the far-reaching ghost hits that a 4× window produced.
- Evidence counts only if the key can still be reused before MRU placement ages out. A ghost hit about one cache's worth of evictions back predicts reuse. A hit from much further back does not, and those keys displace shorter-reuse residents without paying it back. Long-reuse misses beyond about one cache cannot be recovered by stash placement.
- Once the ghost existed, BIP's random MRU path went from the only way out of probation to a net cost. The two admission paths overlap, and the selective one wins.
- Survival of ghost-admitted keys inside the stash is already solved. MRU placement alone protects nearly all of them, so scan-skip protection adds little. This repeats the 0003 finding that stacked retention mechanisms protecting the same items add nothing.
- Gains keep shrinking, and the levers left inside the stash path look close to tuned out.
- For the fourth cycle running, no writer could inspect the overlay source. Evaluation was again hit-rate only, on one LIRS trace at one size. Per-op-time and metadata-size are still unmeasured, even as ghost and flag metadata build up.

## Open questions

- Does pushing `BIP_PERIOD` further, or removing BIP so the ghost is the only route out of probation, keep improving, or does BIP still catch reuse that the ghost window misses?
- Would a ghost window smaller than one cache, or a ghost admission gated on how recently the fingerprint was evicted, sharpen the evidence signal?
- Do these results hold on traces whose long-reuse keys fall outside the ghost window, and at other cache sizes?

## Continuation

Build on v003 (rarer BIP). Probe the BIP endpoint: a still larger period, or no BIP at all, relying on the ghost alone. Optionally try a smaller ghost window or a recency-gated ghost window. Drop ghost windows larger than about one cache and further scan protection for ghost hits. The stash path is nearly tuned out on this trace. The next cycle should put measuring per-op-time and metadata-size and running a second trace or cache size ahead of more hit-rate micro-tuning.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
