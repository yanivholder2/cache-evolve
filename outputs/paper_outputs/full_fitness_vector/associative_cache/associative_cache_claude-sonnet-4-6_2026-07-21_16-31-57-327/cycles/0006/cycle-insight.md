# Cycle 0006 Insight

## Cohort outcome

The cohort probed three independent axes: promotion-budget saturation, bucket-count scaling, and frequency-informed eviction via a write-once binary flag. Both winners stayed on the 3-swap positional base — one confirming its ceiling, the other pairing it with a second-chance tail scan that measurably improved hit rate.

## Patterns

- **3-swap promotion budget is confirmed optimal; the ceiling is now mapped.** Extending to five swaps produced a slight hit-rate regression, meaning items beyond three hops from the head are accessed rarely enough on this Zipfian workload that forcing them forward displaces items that cycle through the top positions. The useful range of promotion-budget exploration is exhausted.
- **Second-chance eviction with a write-once flag is the most promising hit-rate lever found so far.** A binary "ever-accessed" marker — written once at first touch, never again on the hot get path — lets the eviction scan skip past genuinely warm items and eject cold newcomers instead. This directly answers Cycle 0005's call for frequency information without per-get writes.
- **The latency regression in v003 traces to the 3-swap loop, not the flag.** The second-chance scan itself is inexpensive; it is the unconditional position-update swaps on every hit that push per-get write cost above the baseline. The hit-rate gain and the latency cost are therefore separable — they come from different mechanisms.
- **Halving bucket count is a definitively closed direction.** Fewer buckets raise collision density faster than added depth can absorb, and the 3-swap budget cannot rescue items stranded beyond position three in doubled-width buckets. The hit-rate regression replicates Cycle 0001's findings and removes any remaining ambiguity.

## Open questions

- Can the second-chance eviction scan be paired with a cheaper promotion strategy (fewer swaps, or none at all) to preserve the hit-rate lift while eliminating the per-get write regression?
- Does a zero-swap get path — pure positional read with no item movement — combined with the second-chance eviction scan still outperform strict LRU, or does recency ordering degrade too quickly without any promotion?

## Continuation

The next cycle should decouple the two mechanisms in v003: keep the write-once binary flag and second-chance tail scan, but reduce or eliminate the per-get promotion swaps. Specifically, try pairing the second-chance scan with (a) a single swap only — recovering the position-0 guard pattern from Cycle 0003 — and (b) no promotion at all, relying entirely on eviction quality rather than positional convergence for hit-rate. If either pairing holds hit rate near v003's level while restoring latency to the v001 baseline, it becomes the new confirmed base. Bucket count and promotion budget beyond three swaps should not be revisited.
