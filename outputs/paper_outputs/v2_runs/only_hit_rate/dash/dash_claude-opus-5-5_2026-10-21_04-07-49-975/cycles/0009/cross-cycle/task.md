# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants changed only the promotion path out of the stash. One changed where an item lands inside its bucket (v001, MRU landing), one changed which bucket it goes to (v002, spill to a free probing slot), and one added an admission filter (v003, two stash hits before promotion). The winner was v003, the only one that made it harder to get into the normal tier. v002 gained very little, and v001 lowered hit-rate.

## Patterns

- What decides the hit-rate here is which items are allowed into the scarce normal tier, not where they are placed. The variant that made admission stricter won. The variant that made it looser (v001, where promoting to MRU lowers the bar for moving to probing) lost.
- Every eviction in the normal tier starts a cross-bucket demotion cascade into a neighbour's target. Changes that cause more normal-tier moves hurt neighbouring buckets as well as their own.
- In steady state the normal buckets are almost always full together. Changes that only matter when a free slot exists (v002) rarely take effect.
- The stash is larger than the normal tier, so it can hold unproven keys while they wait. That is why a filter at the stash→target edge costs little.
- Item position and promotion thresholds are coupled. The "is hottest" check reads bucket position, so changing where an item is placed also changes how many hits it needs for the next promotion.

## Open questions

- Every variant was scored on hit-rate only. Nothing is known yet about metadata-size or per-op-time, including v003's claimed get-latency improvement.
- Is a threshold of two stash hits the best value? Would a frequency or recency filter on the target→probing edge help in the same way?
- Could an SLRU-style MRU landing work if it were combined with a separate hit count, so the bar for entering probing stays where it is?

## Continuation

Build on v003. Try stricter or adaptive admission on the other edges: a different stash-hit threshold, and a hit-count requirement for target→probing. Get v003 evaluated on per-op-time and metadata-size too. Drop "free slot" placement ideas like v002. Only revisit MRU landing if it keeps the current bar for entering probing.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants were built on cycle-0001 v003. Two of them tightened admission: v001 put a frequency bar on the target→probing edge, and v002 raised the stash→target threshold from 2 to 3. The third, v003, changed retention instead: items demoted from the normal tier now re-enter the stash at the MRU end. v003 won. v002 also raised hit-rate, but by less, and v001 was flat.

## Patterns

- The cohort moved from admission to retention. Last cycle's lesson, "stricter admission wins", now gives smaller returns. The stash edge still gains a little at 3 and looks close to its optimum. The probing edge gains nothing because Front landing already filters it.
- The demotion pre-credit only pays off if the credited item survives long enough to collect its return hit. Where an item is placed in the stash (v003) and how big the credit is relative to the threshold (v002) both decide whether ex-residents come back cheaply. These two mechanisms work together; they are not competing.
- A useful split is "fresh keys face a high bar, proven ex-residents face a low bar and get protected". This is SLRU/2Q-like behaviour, and both variants that gained used it.
- Gating the target→probing edge did nothing on this trace. Its counter accumulates hits and is pre-credited to foreign residents, so it barely filters. The cascade churn from that edge seems small, so there is little left to win there through admission.
- Every result so far comes from a single LIRS trace at a single cache size and covers hit-rate only.

## Open questions

- Do v003 (MRU stash re-entry) and v002 (threshold 3) stack when combined? Or does protecting ex-residents at MRU crowd out one-touch keys that need a third hit?
- Does MRU re-entry hurt on scan-heavy or larger-working-set workloads?
- Per-op-time and metadata-size are still not measured for any winner.
- Would routing evictions differently (where a demoted or cascaded item goes) do better than any further admission gating?

## Continuation

Build on v003. First test combining it with v002's threshold of 3 and the threshold-relative pre-credit. Then probe other stash retention controls, for example the depth at which demoted items land or protection for pre-credited items. Also try routing evictions to change cascade targets. Drop frequency gating on the target→probing edge, and don't push the stash threshold past 3. Get per-op-time and metadata-size measured, and run on at least one more trace or cache size.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants were built on cycle-0002 v003 (MRU stash re-entry) and each tested a different lever. v001 stacked admission by raising the stash threshold to 3. v002 changed stash victim choice to a CLOCK-like scan that prefers counter-0 slots. v003 changed cascade routing so that evicted foreign residents go straight to the stash. None of them moved hit-rate beyond noise. v002 is the nominal winner, but only by a noise-level margin.

## Patterns

- The design looks saturated on this trace at this size. Once the stash works as an MRU-placed victim cache with a pre-credit, admission, victim choice and routing all end up choosing between victims that can be recovered cheaply. Changing which one is chosen barely matters.
- The pre-credited ex-residents seem to dominate stash traffic. That explains all three null results. A higher threshold has no fresh keys to filter (v001). The scan skips the same items that MRU placement already protects (v002). Rerouting the cascade swaps one victim the stash can bring back for another (v003).
- Position-based protection and frequency-based protection in the stash overlap almost completely. Stacking two retention mechanisms that protect the same items adds nothing.
- The neighbour cascade damage flagged in cycle 0001 no longer matters for hit-rate now that the stash recovers demoted items cheaply.
- v001's result matched its parent exactly. That could mean a real no-effect, but it could also mean the change never reached the evaluated build. None of the variant writers could inspect the overlay source.

## Open questions

- Did v001's threshold change actually reach the evaluated build?
- Does v003's cascade-free routing lower per-op-time, as its rationale predicted? It cost no hit-rate, so it may win on that axis instead.
- Is the plateau a property of this trace and cache size, or of the design? Every result so far comes from one LIRS trace at one size.
- Are stash capacity, or the split of slots between the stash and the normal tier, the remaining hit-rate levers?

## Continuation

Stop running hit-rate-only variants on this single trace: three different levers came out flat, so the design looks saturated there. Next cycle should put per-op-time and metadata-size measurement first (v003's cascade-free routing is the natural candidate for a per-op-time win), check that v001's overlay was applied, and evaluate on at least one more trace or cache size. If hit-rate work continues, try structural capacity changes such as the stash-to-normal slot ratio. Drop further tweaks to stash victim choice and admission thresholds.


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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

All three variants were built on cycle-0006 v003 (rarer BIP). v001 removed BIP so that the ghost is the only blind-free route into protected space. v002 added a recency gate to ghost hits, honouring only those from about half the ghost window back. v003 reapplied cascade-free routing of displaced foreign residents to aim at per-op-time. v002 won. v001 also gained a little, but not enough to win. v003 matched its parent's hit-rate exactly, bit for bit.

## Patterns

- Both hit-rate gains came from the same move: tightening what counts as evidence for entering protected stash space. v001 removed the blind path and v002 removed stale evidence from the selective path. Neither added protection. This is the fourth cycle in a row where the zero-sum rule for protected capacity has held.
- Recency matters more than the amount of evidence. In 0006 a larger ghost window lost. This cycle a gate at roughly half a window won. The best ghost horizon looks shorter than one cache's worth of evictions, and direct-mapped overwrite alone does not age entries out quickly enough.
- BIP's optimum looks like "off". On this trace, the reuse it caught beyond the ghost's reach cost more capacity than it returned. Removing it also cuts metadata and per-insert work.
- v001 and v002 are orthogonal (one changes the blind path, the other the selective path), and both were positive on the same parent. They have not been combined yet.
- The cohort's one variant aimed at efficiency produced no signal. An exactly identical result means either the cascade branch never fires at this size or the change did not apply. Per-op-time still has no fitness dimension, so this kind of hypothesis cannot be scored yet.
- For the fifth cycle running, writers could not inspect the overlay source. Every mechanism claim is taken from the hypothesis text and has not been checked against the diff. Evaluation is still one LIRS trace at one size.

## Open questions

- Do v001 and v002 stack (gated ghost with no BIP), or do they reclaim the same protected slots?
- Where is the best gate cutoff: tighter than half the window, or looser? Can the ghost table shrink to fit the gated horizon and save metadata?
- Does the cascade path ever fire at this cache size? A displacement counter would settle whether H03 was a no-op.
- Does removing BIP hurt on traces where useful reuse falls outside the ghost horizon?

## Continuation

Build on v002 (recency-gated ghost). Combine it with v001's BIP removal, then sweep the epoch cutoff and try shrinking the ghost table to the gated horizon. Drop larger ghost windows, extra ghost-hit protection, and any change that adds protection rather than filtering it. Stop proposing per-op-time hypotheses like cascade-free routing until the eval reports per-op-time, metadata-size and a displacement counter. Push again for a second trace or cache size before more micro-tuning.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants were built on cycle-0007 v002 (recency-gated ghost), and each answered one of that cycle's open questions. v001 added cycle-0007 v001's BIP removal, v002 tightened the gate cutoff, and v003 replaced the binary gate with age-graded placement (recent to MRU, moderately old to mid-stash). All three gained. v001 won clearly. v002 and v003 gained only marginally, close to single-trace noise.

## Patterns

- The gate and BIP removal stack instead of overlapping. Removing BIP paid more on the gated parent than on the ungated one. The gate sends stale ghost hits into the fresh-key path, so BIP's blind MRU branch was spending more protected capacity there. Stash MRU now requires evidence: either a recent ghost hit or a demotion. This is the fifth cycle in a row where filtering admission into protected space has beaten adding protection.
- Ghost-recency tuning has reached a plateau. Most of the benefit came from having a gate at all. A tighter cutoff and graded placement each moved hit-rate only slightly, and both acted on the same small band of moderately old ghost hits. Fine-tuning insert position within the stash has stopped paying.
- The winner also simplifies the design. It removes BIP state and a branch on the insert path. The two marginal variants add none of that simplicity, and v003 adds a placement helper.
- For the sixth cycle running, writers could not inspect the overlay source, and every mechanism claim comes from the hypothesis text. Evaluation was still one LIRS trace at one cache size, hit-rate only.

## Open questions

- Does the tighter cutoff (or graded placement) still add anything on top of v001's no-BIP base, or did BIP removal absorb that band?
- Can the ghost table be shrunk to the gated horizon with no hit-rate loss, so metadata drops?
- With no BIP left, does the design lose on traces whose useful reuse falls outside the ghost horizon, or at other cache sizes?

## Continuation

Build on v001 (gated ghost with no BIP). At most, do a single check that combines it with v002's tighter cutoff. Drop further gate-cutoff sweeps and graded mid-stash placement, since they are on a plateau. Make shrinking the ghost table to the gated horizon the main metadata hypothesis. In-stash levers look exhausted on this trace. The next cycle should change who reaches the stash at all, or move evaluation to a second trace or cache size, before doing any more micro-tuning.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

All three variants were built on cycle-0008 v001 (recency-gated ghost, no BIP). Following that cycle's continuation, each one moved off in-stash placement. v001 added an admission doorkeeper that stops fresh keys from evicting evidence-backed residents. v002 halved the ghost table to fit the gated horizon. v003 moved one normal bucket's worth of slots into the stash. v002 won by holding hit-rate within noise (slightly up) while cutting metadata. v001 and v003 both came out flat to slightly negative.

## Patterns

- No hit-rate lever moved this cycle. The admission gate, the stash insert position, and now the tier ratio have all hit a plateau on this trace. The parent's design already does what the doorkeeper and the larger stash were meant to add. Fresh keys land at LRU, and the second-chance scan already keeps evidence-backed residents safe.
- The win was about efficiency, not hit-rate. The recency gate, not the table size, sets how much ghost history is useful. About half a cache's worth of fingerprints holds all the evidence that counts, and the extra direct-mapped collisions did not measurably push out recent entries. This continues the 0007–0008 finding that recency matters more than volume.
- Capacity is roughly fungible between tiers at the current split. A normal slot and a stash slot retain about the same value. So "a stash slot is worth more than a normal slot" is not supported, and nudging the ratio is a weak lever.
- The winner's claimed metadata saving is still unconfirmed. The fitness vector reports only hit-rate, so a variant won on a dimension nobody measured. For the seventh cycle running, writers could not inspect the overlay source, and every mechanism claim comes from the hypothesis text. Evaluation is still one LIRS trace at one cache size.

## Open questions

- Where does ghost shrinkage begin to lose hit-rate (a quarter, an eighth)? Does a set-associative ghost push that point further out?
- Did v003's normal-bucket indexing for a non-power-of-two count spread keys evenly, or did uneven spreading hide a small stash gain?
- Would any of this cycle's flat results change on a second trace or a different cache size, where useful reuse falls outside the gated horizon?

## Continuation

Build on v002 (halved ghost). If you shrink the ghost again, do it only once (for example to a quarter) to find where collisions start to cost hits. Drop the fresh-key doorkeeper and further stash/normal ratio shifts, since both are flat on this trace. Hit-rate micro-tuning on this one trace and size is exhausted. The next cycle should prioritize getting metadata-size and per-op-time reported, and should evaluate on a second trace or cache size, before proposing new mechanisms.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
