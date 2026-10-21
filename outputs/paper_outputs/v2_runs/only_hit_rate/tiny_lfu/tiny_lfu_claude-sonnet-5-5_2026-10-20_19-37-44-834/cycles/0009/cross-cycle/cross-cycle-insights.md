## Recurring wins

- Changes that preserve frequency information and leave admission semantics alone are neutral at worst (index reuse, single-hash indexing, slab links, flat u8 packing). They are candidate free cost wins, but none has been verified on size or speed.
- Loosening the window-to-probation duel to tie-admit (cycle 3) is the only admission change that ever clearly helped. Everything after it was flat or negative.
- Sketch width moves hit-rate monotonically with sharply diminishing returns. Wider rows (4x, and width 8) gained slightly, and narrower rows lost.
- Count semantics and the admit threshold are coupled. Retuned together they restore parity (cycle 8), but not more.

## Recurring failures

- Anything that degrades frequency estimates lost hit-rate: narrower rows, 4-bit saturating counters, correlated or blocked rows, and a doorkeeper.
- Decay and aging (halving, even very infrequent) never paid off. The workload is stable and the sketch is not decay-limited.
- Admission-rule tuning (looser than tie-admit, recency tie-breaks, standalone single-count) is flat or slightly negative.
- Segment-internal tuning (protected size, demotion choice, probation victim sampling) is flat. Window-size tuning and hill-climbing are also flat.
- Sketch depth (an extra row) is flat.
- "Winners" are often near-ties or slight losers, chosen on hit-rate alone among noise-level differences.
- Evaluation is the blocker. In every cycle so far the fitness vectors report hit-rate only. Metadata-size and per-op-time are unmeasured, and only one trace and cache size is used.
- Hypothesis text is sometimes muddled about scaling direction or parent. Variant authors usually cannot read the overlay source, so the proposals are generic.

## Open questions

- Do the cost-oriented changes (single-hash, index reuse, slab links, flat u8, narrower or wider sketch) actually reduce size or per-op-time? This cannot be answered until those dimensions are measured.
- Is workload-a saturated for hit-rate? A second trace with shifting popularity, or a different cache size, is needed. It would also give aging, the doorkeeper, and adaptive windows a fair test.
- Is the width trade-off worth its size cost? Compare intermediate widths against baseline on size.
- Would 4-bit counters recover with decorrelated hashes? Would blocked rows do the same? Both need cost measurement.
- Does a per-entry flag, such as the touched-in-window bit, cost more memory than its tiny gain?
- Eviction or segment structure, rather than sketch and admission, may be the only untouched lever.

## Per-policy notes

- TinyLFU-style windowed design (window + segmented main + count-min sketch with sketch-gated admission): the tie-admit duel and the single-count with lower threshold are the current base. Hit-rate on workload-a is near its ceiling.
- Sketch: u8 counters with conservative update are safe. 4-bit counters, blocking, and width cuts hurt. Wider helps slightly. Single-hash derivation is behavior-safe.
- Admission: sketch estimate dominates, and in-window recency adds nothing beyond it.
- Segments: protected/probation sizing and victim choice are irrelevant on this trace.
- Before the next hypotheses, fix the fitness output (size, time, second trace). Then verify the cheap behavior-preserving changes on cost, and state the parent and scaling direction in every width or packing hypothesis.
