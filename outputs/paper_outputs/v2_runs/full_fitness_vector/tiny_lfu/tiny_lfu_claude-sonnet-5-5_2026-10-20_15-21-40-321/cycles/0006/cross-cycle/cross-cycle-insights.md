## Recurring wins

- Removing repeated hashing from the sketch path (single hash, cheaper mixing, cached hash) was the clearest early speed gain, mostly on put tail. It held up on the baseline lineage but did not stack on stronger lineages (cycle 0006).
- Replacing the map hasher (with MRU short-circuit on hit) was the largest later latency win. It recovered the per-op cost of the compact link layout and kept that layout's size gain, with hit-rate unchanged.
- Compact layouts (u32 links, u8/nibble counters) cut footprint. Without extra hot-path work they leave hit-rate flat. Sketch heap savings are invisible in the headline metadata number, which tracks Node size.
- Changes that make existing machinery cheaper, rather than adding mechanism, are the only ones that have paid off.

## Recurring failures

- Hit-rate levers on workload-a have all failed: aging (twice, including a 10x longer period), small window, wider victim sample, tie-admitting `>=`, doorkeeper, wider sketch, conservative update. Strict greater-than admission with the LRU head as victim fits this trace (low, often tied, stable frequencies).
- Adding state or hot-path work (doorkeeper bitset, window, segment lookup, halving counters) raises tail latency and grows the struct without a benefit.
- Sketch tweaks of every kind (accuracy, width, increment cost, early-exit) are exhausted. Counters rarely saturate, so the early-exit rarely fires.
- Node field removal did not shrink the struct because alignment absorbed it, and it moved work onto the eviction path.
- Small per-op deltas, including ones in untouched code, recur as noise. Several apparent gains and losses are not trustworthy without repeat runs.
- Shrinking sketch precision too far (4-bit, one row, tight sizing) costs hit-rate noticeably.
- Many insights were written without a source diff, so attributions are inferred.

## Open questions

- Where does `get` spend its time on the incumbent: map lookup, list relinking, sketch estimate, allocation, bounds checks? No profile exists.
- Are the incumbents at the floor for this design?
- Is there any hit-rate lever left on workload-a? A structurally different mechanism is needed (segmented layout, larger window), or a trace with drift to justify decay.
- Which part of the early single-hash gain was real, and why did it not stack on the stronger lineage?
- Can the compact nibble sketch recover its hit-rate loss cheaply, and does it combine with the cheap map hasher?
- How much latency variance is noise? Repeat or larger measurements are needed.

## Per-policy notes

- TinyLFU (the only policy so far): the cycle-0004 v001 lineage (cheap map hasher, MRU skip, compact links) and the cycle-0004 v003 incumbent are the current parents. Baseline-rooted variants have shown gains that did not transfer to them.
- Admission by sketch estimate, strictly greater than the LRU-head victim, has resisted every change. Treat it as fixed unless a new mechanism is proposed.
- Next hypotheses should target the non-sketch hot path or a structurally different admission design, and name the cost they expect to move.
