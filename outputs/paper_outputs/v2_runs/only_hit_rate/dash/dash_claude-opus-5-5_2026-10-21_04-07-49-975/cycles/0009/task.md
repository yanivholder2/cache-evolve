# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0009

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0009/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
