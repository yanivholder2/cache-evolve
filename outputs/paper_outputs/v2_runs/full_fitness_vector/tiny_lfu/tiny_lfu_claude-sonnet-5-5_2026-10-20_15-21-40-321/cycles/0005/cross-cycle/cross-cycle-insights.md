## Recurring wins

- Cutting per-op hashing cost is the most reliable speed lever: single hash, shift/bit-sliced index derivation, cheap map hasher, saturation early-exit. It has paid off in cycles 0001, 0002, 0004 and 0005 on independent lineages, with hit-rate left flat.
- Changes that make existing machinery cheaper without altering counters, update rule or admission keep hit-rate flat. Changes that add mechanism rarely help.
- Compact layout (u32 links) cuts Node size. Its latency cost was recovered by the cheap map hasher (0004).

## Recurring failures

- Aging or decay (naive, long-period): workload-a has stable frequencies, so decay discards useful history. Failed in 0001 and 0002.
- Admission-path additions: window, doorkeeper, wider victim sample, tie-admitting `>=`. None lifted hit-rate. Several raised tail latency or lowered hit-rate. Ties are common and loosening protection hurts.
- Sketch width/precision tuning: widening gives marginal hit-rate at a large hidden heap cost, and narrowing gives a hair of loss with no speedup. 4-bit counters trade hit-rate for footprint. Sketch accuracy looks saturated.
- Node field removal: padding/alignment absorbed the saving, and recomputation taxed put tail latency.
- Extra state on the hot path grows the struct and widens the tail.
- Small latency deltas (a few percent, or gains on untouched paths) have repeatedly proved to be noise. Do not credit them without repeat runs.

## Open questions

- Does the 0005 v003 shift-derived increment stack on the 0004 v001 lineage (cheap hasher, MRU skip, compact links)? Which part matters: early-exit or hash derivation?
- Can the nibble sketch plus fast hashing be combined, recovering hit-rate with minimal extra accuracy?
- Is any hit-rate lever left on workload-a? It would need a genuinely new mechanism, not a tweak to admission, victim choice, sketch size or aging.
- Is there a drifting trace that would justify decay or a window?
- How large is the latency noise floor, and is repeated measurement needed?
- Is there any footprint lever left once Node alignment is the floor?

## Per-policy notes

- TinyLFU (the only policy so far, tested on workload-a): the baseline's strict greater-than admission, LRU-head victim, never-decaying counters and immediate admission are close to a local optimum for hit-rate. Gains have come from per-op cost and footprint.
- The metadata headline tracks Node size, not sketch heap, so sketch savings are invisible there. Judge sketch changes separately.
- Several insights were written without source diffs, so attributions (for example hash caching versus mixing) are inferred.
