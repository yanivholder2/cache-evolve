## Recurring wins

- Making the frequency sketch cheaper (fewer hashes, cheaper index derivation, smaller counters) is the only direction that has paid off. Put-tail latency improves most.
- Changes that leave counters, conservative update and admission semantics untouched keep hit-rate flat while cutting cost. Single-hash plus conservative update (cycle 1) and the single-mix bit-sliced hash (cycle 2) are examples.
- Hashing and index derivation dominate per-op cost on workload-a. Cutting them is the most reliable lever.

## Recurring failures

- Aging or decay of any kind failed in both cycles, including a period 10x longer. workload-a has stable frequencies, so decay discards useful history. Without a drifting trace it has no case.
- Adding mechanism to the admission path (small window, trial period) gained no hit-rate and cost time and metadata.
- Aggressive precision cuts (4-bit counters, one row, tight sizing) buy speed and footprint but visibly lose hit-rate.
- Extra state or bookkeeping on the hot path or in the cache struct adds latency and grows the struct without a matching benefit.

## Open questions

- How much of the nibble layout's hit-rate loss comes from the 4-bit cap, the single row, or the tight sizing? What is the cheapest fix (second hash or larger table)?
- Does the cached-hash speedup combine with the compact layout, or does its extra state offset the footprint gain?
- Which parts of the cycle-1 sketch rewrite (single hash, u8 counters, conservative update) account for its gains?
- Do the u8 cap and conservative update hold on traces with larger frequency gaps or drift?
- Is there any trace property that would justify aging, a larger window or a doorkeeper?
- How much of the cached-hash attribution is real? It was inferred without source access.

## Per-policy notes

- TinyLFU (sketch plus strict-greater admission): the baseline admission rule and never-decaying counters suit workload-a. Improvement room is in sketch cost and accuracy, not in admission mechanics.
- The headline metadata number tracks Node size, not sketch heap. Sketch savings are invisible there, and added struct fields show up as small growth.
- Preferred parent for speed: the cycle-2 single-mix hash variant, which has no hit-rate cost. Next step is to combine it with the compact layout plus minimal extra accuracy.
