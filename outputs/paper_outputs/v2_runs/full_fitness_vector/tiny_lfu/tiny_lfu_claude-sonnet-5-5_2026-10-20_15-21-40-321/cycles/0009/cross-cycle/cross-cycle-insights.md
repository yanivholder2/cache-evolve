## Recurring wins

- Cutting per-op hashing cost on the baseline lineage pays: single-hash sketch (c0001), cheaper index derivation (c0002), cheap map hasher (c0004), and the flat table with one hash reused for the sketch (c0009, the largest win). The map and hashing path is the dominant per-op cost.
- Storage-layer changes (map hasher, flat table over a slab) move latency while leaving hit-rate and policy untouched. These are the safest speed gains.
- Gains tend to appear on the baseline and then fail to transfer to stronger lineages (c0005 to c0006). Test stacking explicitly.

## Recurring failures

- Hit-rate levers on workload-a have all been flat or negative: aging (twice), small and smaller windows, victim sampling, tie-admitting, doorkeeper, wider sketch, conservative update, segmented main LRU. The baseline's strict-greater admission with the LRU head as victim looks near the ceiling.
- Sketch-internal tweaks (width, accuracy, early-exit, hash derivation) are exhausted. Sketch accuracy is saturated.
- Footprint cuts (compact Node, narrower or halved sketch, field removal) usually do not become speed gains. They often add conversion or unpacking cost, and alignment can absorb the saving. The working set is small, so locality does not help.
- Extra hot-path state or bookkeeping (doorkeeper bitset, victim cache, cached fields, inline hints) raises tail latency and struct size.
- Pre-sizing, inline and branch hints, and relink skipping did nothing. Allocation and list relinking are not the cost.
- Sketch heap changes are invisible in the headline metadata number, which tracks Node size and is dominated by the sketch.

## Open questions

- Does the c0009 flat-table and single-hash gain stack on the best hit-rate incumbents? Which part carries it: the table or the hash reuse?
- Is any hit-rate lever left on workload-a? The adaptive hill-climbing window was never evaluated because of an implementer failure.
- Is there a profile of where get and put spend time? Most hot-path guesses have been blind.
- How much latency delta is noise? Repeat runs are needed before crediting small deltas, and several earlier "gains" were likely noise.

## Per-policy notes

- tiny_lfu (window + sketch admission + LRU main): sketch admission already gives the scan resistance this trace needs. Hit-rate is robust to every structural and sketch variation tried.
- Winner status often reflects the selection rule, not real improvement. Several kept variants were regressions or flat.
- Several insights were written without source diffs, so some attributions are inferred. Audit diffs for unexplained size or latency changes before building on a variant (e.g. c0008 v002).
- Next direction: port the c0009 flat-table storage onto the best incumbent, and target the map and hashing path rather than the sketch or list.
