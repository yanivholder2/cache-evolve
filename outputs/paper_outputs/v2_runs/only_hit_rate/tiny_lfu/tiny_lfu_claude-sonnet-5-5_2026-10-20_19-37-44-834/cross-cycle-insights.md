## Recurring wins

- Changes that preserve frequency information and loosen admission only slightly (cycle 3 tie-admit duel) are the only admission change that helped. Behavior-preserving changes (index reuse, slab links, single-hash) match the parent exactly.
- Wider sketch rows help slightly and monotonically (cycles 7, 8, 10), with sharply diminishing returns.
- Count semantics and the admit threshold must be tuned together. Retuning both restored parity (cycles 7-8).

## Recurring failures

- Anything that discards or degrades frequency information loses hit-rate: aging/halving, larger window, doorkeeper, narrower rows (any size), 4-bit saturating counters, blocked/correlated rows.
- Admission-rule tweaks beyond the tie-admit duel (looser gating, tie-break flags, ghost bonus, extra rows) are flat at noise level.
- Segment-internal tuning (protected size, demotion choice, probation victim sampling) is flat.
- Adaptive window tuning has no value on a single stable trace.
- Winners are often near-ties or least-bad losers, chosen on hit-rate alone.
- For ten cycles, fitness has reported only hit-rate. Metadata-size and per-op-time have never been measured, so no cost idea has been judged. Hypotheses are also muddled (direction of width scaling, parent) and authors cannot read overlay source.
- The one rejection (cycle 1 aging) was environmental (HTTP 429 on a dependency download), not a flaw in the idea.

## Open questions

- Do the cost changes (flat u8 packing, single-hash, index reuse, slab links, compact sketches) actually shrink metadata or speed up ops? This is the main unverified thread.
- Is workload-a saturated for hit-rate? A second trace (shifting popularity or different cache size) is needed, and might revive aging, doorkeeper, or adaptive-window ideas.
- Does an intermediate width (such as 8) keep most of the wide-sketch gain at lower size?
- Could a blocked sketch with decorrelated rows keep hit-rate and gain locality?
- Is eviction or segment structure the only remaining hit-rate lever?

## Per-policy notes

- TinyLFU / W-TinyLFU (window + segmented main + sketch-gated admission): the baseline is near its optimum on workload-a. The sketch is mildly collision-limited and not over-narrowable; hash-sharing is safe. Keep the tie-admit duel and single-count with a lower threshold as the base.
- Next steps: fix the evaluation first (add metadata-size and per-op-time, add a second trace). Keep cost and behavior changes in separate variants. Any width hypothesis must state its parent and scaling direction.
