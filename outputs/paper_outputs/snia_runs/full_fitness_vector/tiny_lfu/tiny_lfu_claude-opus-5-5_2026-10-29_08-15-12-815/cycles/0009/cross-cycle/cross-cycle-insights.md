# Cross-Cycle Insights (through cycle 0009)

## Recurring wins

- Merging orthogonal winners keeps adding up. Fast hash + W-TinyLFU (0002), aging sketch + window/SLRU (0003), the sketch on the 24-byte node (0004), and window floor + pairwise conservative update + u32 links (0006) all stacked without interference. Merges stop adding up only when two changes act on the same mechanism (see failures).
- Bit/word packing that removes stores or memory pays off. Sentinel-tagged links, nibble-packed 4-bit counters and the cache-line-blocked sketch each improved metadata and latency together (0002–0003). This holds only when the packed field isn't written on the hot path (see failures).
- Conservative update is the large-size accuracy lever (0003–0005). It fixes correlated in-block overestimation, and the pairwise approximation keeps nearly all of the gain. An exact global min is unnecessary.
- An absolute window floor is the small-size lever (0004–0006). It leaves the large size untouched. Small and large sizes keep wanting opposite things, so levers that touch only one size are the safe kind.
- Latency gains now come from codegen, not algorithm changes. These include u32-only index arithmetic with no bounds checks and single-store links (0005), and `#[inline(always)]` on `increment_hashed` (0006). The supposed conservative-update latency cost was an out-of-line-call artifact.
- A small, explicit, bounded bonus for re-seen candidates works: the `>= 2`-guarded tie-break on the dedup line (0008 v002). It is the current hit-rate base.

## Recurring failures

- Explaining get p99 as a pointer-chase or link-write cost has failed four times: sketch prefetch, neighbour prefetch, an in-link CLOCK bit, and a bitset CLOCK bit (0007–0009). Protected-segment CLOCK also hurts hit-rate. Drop that line of attack and look at the inlined sketch update and the map probe instead.
- Software prefetch on the inlined merged line overlaps with the inlining gain instead of adding to it (0007).
- Changing the sketch layout cannot fix overestimation. In-block row addressing gained nothing (0006), and the fixed row-to-word-pair layout is valuable because its rows never alias.
- Latency fixes that only remove branches or shorten the `min` chain did nothing (0004–0005).
- Stacking candidate bonuses doesn't work. The double count, the tie-break and the restored aging clock all fix the same large-size newcomer loss. Combining any two adds nothing at the large size and over-favours recency at the small size (0009).
- Uniform knobs fail because the two sizes pull in opposite directions. A uniform larger window percentage (0004) and a uniform `tick()` (0009) each helped one size and hurt the other.
- Runtime hill-climbing window adaptivity (0002) cost latency on every op for a gain only at the small size.
- Flags packed into link words turn blind relink stores into load-then-store sequences (0008).

## Open questions

- Does a size-dependent aging rate (slower at small capacity) keep the small-size gain while helping the large size? The small-size gain since 0008 traces back to slower aging, not to the tie-break.
- Should the tie-break threshold scale with capacity or sketch width instead of being a fixed constant?
- What actually drives get p99? Profile the inlined sketch update and the map probe.
- How large is the per-op-time noise floor? Single-run latency verdicts flipped on byte-identical hot paths in 0004, 0005 and 0007–0009. Judge latency only from interleaved or repeated A/B runs.
- Do the gains generalise across traces? The window floor's gain was hm_0-specific, and web_0 is the most sensitive cell at the large size.

## Per-policy notes

- W-TinyLFU window + SLRU: this is the residency backbone. The 24-byte u32-link node is now as fast as the 40-byte one or faster, so build on it. Keep protected as a true LRU (move-to-tail), not CLOCK. Size the window with an absolute floor, not a percentage, and don't use runtime adaptivity.
- Count-min / frequency sketch: use the cache-line-blocked layout with 4-bit counters, a fixed row-to-word-pair layout, pairwise conservative update and forced-inline increments. Don't tune `SAMPLE_FACTOR` or counter width uniformly. Any change to how often counts are added also changes the reset rate, so the two are coupled.
- Admission: use the dedup line (no get→put double count, no uniform tick) with a `>= 2`-guarded tie-break. The old double count was an implicit recency bonus. Use only one candidate bonus at a time.
- CLOCK / reference-bit variants: both in-link and side-bitset forms regressed latency and hit-rate. Drop them.
