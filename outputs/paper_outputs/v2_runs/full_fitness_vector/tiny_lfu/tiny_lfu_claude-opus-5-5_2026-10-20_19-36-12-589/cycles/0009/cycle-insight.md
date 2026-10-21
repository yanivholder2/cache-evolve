# Cycle 0009 Insight

## Cohort outcome

All three variants went after cycle 0008's claim that the sketch's per-access memory traffic was the main `get` cost. v001 narrowed counters from `u16` to `u8`, v002 software-prefetched the counter lines ahead of the map probe, and v003 put each key's four counters into one 64-byte block. All three are listed as Winners, but only v001 is a real step forward. It halves sketch memory, trims `put` p50 and both tails, and keeps hit rate unchanged. v002 and v003 both raised median latency, and v003 also lowered hit rate.

## Patterns

- Cycle 0008's memory-traffic diagnosis was mostly wrong at this size. The sketch stays in L1/L2 under the benchmark. Halving it left `get` p50 flat, and the two changes built to hide or remove misses only added ALU work to the critical path: extra index math and prefetches in v002, an extra `fmix64` in v003. Making the sketch smaller helps a little. Hiding latency or relaying out the table does not.
- Any fixed cost on the per-access sketch path shows up at the median. A second `indices()`, an extra hash mix or extra prefetch instructions each cost more than the miss savings they targeted. Cycle 0007's lesson about fixed overhead beating rare-path savings applies here too.
- Counter range was wasted, but the independence of counters across rows was not. Saturating at 255 never changed a duel, which again shows admission is decided by low counts. Blocking counters together, even with disjoint lanes, ties the rows to each other and inflates cold keys' estimates. That is the second time a blocked layout has failed, after cycle 0003.
- `put` gains more than `get` from sketch shrinkage, because a full-miss `put` walks the sketch twice. `get` p50 has now resisted changes to relinking, hashing, sketch size and sketch layout.

## Open questions

- Would packed 4-bit counters (saturation at 15) keep the duel intact and shrink metadata further, or would the nibble masking cost the median more than the smaller footprint saves?
- Is `get` p50 at its floor for a SwissTable lookup plus one sketch increment, given that hash-count reductions also failed to move it?

## Continuation

Adopt v001 (CLOCK with saturating `u8` counters) as the new base. Drop software prefetching and all blocked or one-line sketch layouts. Don't add hash mixes or index recomputation to the sketch path. A 4-bit counter variant is the one sketch lever still worth trying: require the hit rate to stay exactly the same and judge it by medians repeated side by side against v001. Otherwise, treat `get` p50 and hit rate as near their ceilings, and go for metadata reductions that preserve behaviour.
