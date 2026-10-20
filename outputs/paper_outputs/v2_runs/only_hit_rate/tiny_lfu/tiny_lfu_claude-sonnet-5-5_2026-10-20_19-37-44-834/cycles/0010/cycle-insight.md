# Cycle 0010 Insight

## Cohort outcome

Three changes were tried: a 2x wider sketch on top of the prior base (v001), a ghost ring giving recently rejected candidates a one-count bonus (v002), and a 2x wider flat-u8 sketch with double-hashed indices (v003). v001 was named Winner, but all three moved hit-rate only at noise level, so the pick is effectively a tie.

## Patterns

- Two of three variants were width changes. Both gained marginally, which repeats the earlier finding that width has sharply diminishing returns and the sketch is not collision-limited on workload-a.
- The ghost ring (a nudge to the same frequency estimate) did nothing. Small adjustments to admission inputs stay flat. This is the latest of many flat admission-rule results.
- v003 mixed a hit-rate change with a cost change (u8 packing, fewer hashes), so even if size or speed improved, it could not be separated from the width effect. Its cost claims are unverified.
- For the tenth cycle running, fitness reports hit-rate only, with no metadata-size or per-op-time. Winners keep being chosen as near-ties on hit-rate alone.
- Variant authors again could not read overlay source, so explanations rest on hypothesis text and fitness vectors.

## Open questions

- Does the u8 packing offset the cost of a wider table? That needs a size measurement.
- Is workload-a saturated for any sketch or admission change, so that only eviction structure or a different trace can show movement?

## Continuation

Stop proposing sketch width, depth, ghost or bonus-count, and admission-rule hypotheses. All are flat across many cycles. Before generating more hypotheses, add metadata-size and per-op-time to the fitness output, plus a second trace (shifting popularity or a different cache size). Then judge v003's packing and the wide-sketch variants on cost. Any further hypothesis should target eviction or segment structure, and cost changes should be kept separate from behavior changes.
