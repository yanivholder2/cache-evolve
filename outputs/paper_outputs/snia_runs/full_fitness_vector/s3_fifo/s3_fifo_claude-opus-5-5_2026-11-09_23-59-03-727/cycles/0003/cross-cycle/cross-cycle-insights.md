## Recurring wins

- One change per variant. Cycles 1–2 bundled changes or moved a single trade-off knob, and they kept trading hm_0 gains for web_0 losses. Cycle 3 isolated each change and got the first cohort with no regression on either trace.
- Taking away "credit without reuse" helps hm_0 and doesn't hurt web_0. Examples: ghosting only main victims that were hit while resident, and resetting freq on promotion. Filters that free ghost budget also lengthen the useful history at no cost.
- Structural implementation changes earn timing credit and leave hit rates untouched. Two have worked so far: slab plus hash indexes (cycle 2), then a faster Fx-style hasher (cycle 3). They are the only per-op-time gains that their diffs explain.
- Per-entry metadata has stayed flat in every cycle. Policy and structural changes haven't added per-entry cost so far.

## Recurring failures

- Fixed-ratio queue sweeps and resizing ghost regions only move the hm_0 vs. web_0-large trade-off around. The summary score never improves (cycles 1–2).
- The ARC-style adaptive small/main split regressed. Its signals were unbalanced and the ghost bound was tied to a moving main size (cycle 1).
- Loosening graduation to `freq >= 1` hurts web_0. Cycle 3 confirmed the threshold as the cause, and the freq reset was cleared. Keep `freq > 1`.
- Per-op-time from policy-only diffs is noise. Unexplained timing shifts, in both directions, have shown up in all three cycles. Don't give timing credit without a structural cause.
- Both early hypotheses about web_0's mechanism were wrong: ghost-slot competition and the threshold-only effect. This happened because changes were bundled.

## Open questions

- Do the cycle-3 Winners stack additively once merged? They are Fx hashing, reset-on-promotion, and reuse-gated main-victim ghosting.
- What causes the rest of web_0's large-size gap? Candidates: the ghost bound relative to main size (the hidden coupled variable), or the small-queue share. Test one variable at a time.
- Can the benchmark environment be pinned, with a same-machine baseline re-run, so that timing for policy diffs becomes trustworthy?
- Should heap bytes per resident key supplement the per-entry metadata metric? Hashed structures and ghost key storage add memory that the current metric doesn't see.
- Further structural speedups, e.g. using the entry API to cut hash probes per miss.

## Per-policy notes

- S3-FIFO (the only policy explored so far):
  - hm_0 consistently rewards more main residency and history for reused keys.
  - web_0, especially at the large size, is sensitive to looser graduation and to squeezing the probation window or the demotee history.
  - The ghost bound is shared, zero-sum, and coupled to main size. Changes that alter main size or ghost contents without accounting for this have backfired.
  - The current best base is slab plus hash, with Fx hashing, the `freq > 1` threshold, reset-on-promotion, and reuse-gated main-victim ghosts. That combination is still to be confirmed as additive.
