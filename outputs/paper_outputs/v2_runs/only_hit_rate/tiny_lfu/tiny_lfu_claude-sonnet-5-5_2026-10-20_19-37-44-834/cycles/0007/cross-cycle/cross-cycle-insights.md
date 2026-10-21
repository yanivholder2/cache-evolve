## Recurring wins

- Changes that preserve or improve frequency-estimate quality are the only ones that ever help: the tie-admit duel (cycle 3) and wider sketch rows (cycle 7). Gains are small.
- Behavior-preserving cost refactors (single-hash indexing, index reuse, slab links) match the parent on hit-rate. They are safe candidates, but their size and speed benefits have never been measured.
- Sketch width moves hit-rate monotonically, with sharply diminishing returns: narrower lost (cycles 4 and 5) and wider gained slightly (cycle 7).

## Recurring failures

- Anything that degrades frequency information loses hit-rate: aging/decay, a doorkeeper, 4-bit counters with a shared hash, narrower rows, blocked rows with correlated slots, and admission looser than ties.
- Segment-internal tuning is flat: protected size, demotion victim, and probation victim sampling. Winners from these are ties or least-bad among losers (cycles 2, 4, 5, 6).
- Window-size changes (a larger window, hill-climbing) give nothing on a single stable trace.
- Every cycle's fitness reports hit-rate only. Metadata-size and per-op-time have been unmeasured for seven cycles, so Winner choices rest on noise-level hit-rate differences.
- Changing count semantics alone (single-count) hurt, because it is coupled to the admission threshold.

## Open questions

- Does the evaluation ever report metadata-size and per-op-time? It must, before more hypotheses are generated.
- Is workload-a saturated for hit-rate? A second trace (shifting popularity or a different cache size) is needed to find out, and to judge aging, the doorkeeper, and adaptive window ideas fairly.
- Is the wider-row gain worth the metadata cost? Try an intermediate width.
- Would single-counting work if the tie-admit threshold were retuned with it?
- Can the compact 4-bit sketch recover hit-rate with aging and decorrelated rows? Does blocking work with per-row decorrelation?
- Does a never-decaying sketch limit hit-rate, so that the reset period is the real lever?

## Per-policy notes

- TinyLFU-style window + segmented main + count-min sketch (the only lineage): the baseline's sketch-gated admission captures most of the benefit. Remaining variation is within noise.
- Admission: tie-admit is the current limit. Looser rules swap in colder entries.
- Sketch: the size is near the collision threshold. Keep the u8/conservative-update design. Hash-sharing and index reuse are likely safe cost wins, pending speed measurement.
- Segments: protected/probation internals are irrelevant to hit-rate here. Drop further tuning.
