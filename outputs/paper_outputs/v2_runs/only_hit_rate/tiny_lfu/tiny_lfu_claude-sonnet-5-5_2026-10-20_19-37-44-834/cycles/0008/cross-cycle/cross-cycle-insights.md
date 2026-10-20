## Recurring wins

- Changes that preserve or improve frequency-estimate quality are neutral-to-slightly-positive: wider sketch rows (cycles 7, 8), the tie-admit duel (cycle 3), and behavior-preserving rewrites (single-hash indexing, index reuse, slab links).
- Width moves hit-rate monotonically with sharply diminishing returns. Wider helps slightly and narrower hurts.
- Count semantics and the admit threshold are coupled. Retuning them together restored parity (cycle 8).

## Recurring failures

- Changes that discard or degrade frequency information lose hit-rate. Examples: aging or halving at any period, doorkeeper, narrower rows, 4-bit saturating counters with a shared hash, blocked correlated rows, and admission looser than the tie-admit duel.
- Segment-internal tuning is flat: window size (including adaptive), protected share, demotion choice, and probation victim sampling. Workload-a appears saturated and stable, so these only produce noise.
- Winners are repeatedly ties or least-bad variants chosen on hit-rate among near-equals, not real gains.
- Evaluation has been blind for eight cycles. Fitness reports hit-rate only, with no metadata-size or per-op-time, and only one trace (workload-a) at one cache size. The cost-oriented ideas have never been judged on their actual goal.
- Variant authors cannot read overlay source, so explanations are unconfirmed. Some hypothesis text has muddled the direction of a width factor.

## Open questions

- Do the cost candidates (single-hash, index reuse, slab links, single-count with a lower threshold, width 8 vs 16) actually save size or time? This needs metadata-size and per-op-time in the fitness output.
- Would a second trace (shifting popularity, a different cache size) reveal value in aging, a doorkeeper, or adaptive windows that workload-a hides?
- Is workload-a saturated for hit-rate, so that only cost work remains?
- Can blocked or compact sketches work with decorrelated per-row hashes?
- What is the best width/size trade-off point?

## Per-policy notes

- TinyLFU, the only policy so far, is a window plus segmented main region, a sketch-gated admission duel, and a sketch with conservative update. A clean base is the tie-admit duel with single-count and a lower threshold on the baseline-or-wider sketch.
- Drop: aging and halving, doorkeeper, window tuning, protected sizing, demotion and probation victim tweaks, width cuts, and further admission loosening.
- Keep as unverified cost candidates: single-hash indexing, index reuse, slab links, and intermediate width.
- Any width hypothesis must state explicitly which direction its factor scales the sketch.
