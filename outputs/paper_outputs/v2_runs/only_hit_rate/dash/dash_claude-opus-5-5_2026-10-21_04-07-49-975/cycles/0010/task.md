# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0010

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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

Exactly one file: `cycle-insight.md` in the output dir.
