## Recurring wins

- Merging winners from separate axes has added up every time (cycles 2–5). Residency, sketch storage, link layout and update rule have not interfered with each other. Merge first, then add new mechanisms.
- Packing bits keeps paying off. Sentinel-tagged links, nibble-packed 4-bit counters and u32 indices each cut metadata at little or no hit-rate cost. Once the codegen was fixed, the 24-byte node also costs nothing in latency.
- Cache-line-blocked sketch layout fixed the scattered-miss get tail (cycle 2). It has held up as the sketch base since.
- Most hit-rate gains come at the small cache size: window/SLRU, aging, a larger or floored window, and the adaptive window all gain there. The exception is conservative update, the one lever that reliably helps the large size, by fixing correlated in-block overestimation.
- Approximate conservative update (pairwise min) keeps nearly all the accuracy of the exact min. Exactness in the sketch update rule is not worth paying for.

## Recurring failures

- Trying to cut latency in the sketch increment, whether by removing branches (cycle 4) or shortening the `min` chain (cycle 5), has not recovered per-op-time. The diagnosis that the serial min chain sets latency has been tried and dropped. Stop trying variants of this.
- Runtime adaptivity (hill-climbing window) cost latency on every op and gained only at the small size. A static size-aware window floor gets that gain more cheaply.
- Uniform changes to the window percentage, or changes that forget or bypass frequency evidence, keep causing small slips at the large cache size.
- Tuning `SAMPLE_FACTOR` or counter width was ruled out as the fix for the large-cache slip, because conservative update explains and fixes it.
- Per-op-time verdicts have kept turning out unreliable (cycles 3–5). Byte-identical hot paths have shown slowdowns, so single-run latency deltas are near the benchmark's noise floor.

## Open questions

- Benchmark stability: do the latency regressions in cycle 5's v002 and v003 survive interleaved, repeated A/B runs against their parents?
- Does adding conservative update (full or pairwise) stop `increment_hashed` from being inlined into `get`/`put`? Is that the real source of its latency cost?
- The hit-rate mismatch between the 24-byte and 40-byte lines' sketches (open since cycle 4) needs an answer before the lines are merged.
- Why is the window floor's small-size gain specific to the trace (hm_0 gains, web_0 barely moves)?
- Is there any lever left that gains hit-rate at the large size beyond conservative update?
- Costs not measured yet: the per-entry map hash and growth of the fixed cache struct are missing from the metadata figures.

## Per-policy notes

- W-TinyLFU (window + SLRU main): this is the core line. It has been stable under every merge since cycle 2. The window should be sized as an absolute floor (max of a percentage and an entry count), not a uniform percentage.
- Count-min / TinyLFU sketch: the blocked 4-bit aging design with a single cheap hash is the settled base. Conservative update (pairwise form) is the preferred update rule. The increment is where per-op cost sits, but the cause looks like codegen or inlining, not arithmetic.
- Residency layer (linked SLRU nodes): the 24-byte u32-link layout with tagged single-store links and bounds-check-free indexing is now the base. Its earlier slowdown was link codegen and is fixed. The `NIL` sentinel on `next` and a prehashed map key are part of this line.
- Plain admit-or-drop baseline with a never-aging sketch: superseded since cycle 1. Do not return to it.
