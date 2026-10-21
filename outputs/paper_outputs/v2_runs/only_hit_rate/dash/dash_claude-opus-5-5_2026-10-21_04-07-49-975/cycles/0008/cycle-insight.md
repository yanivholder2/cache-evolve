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
