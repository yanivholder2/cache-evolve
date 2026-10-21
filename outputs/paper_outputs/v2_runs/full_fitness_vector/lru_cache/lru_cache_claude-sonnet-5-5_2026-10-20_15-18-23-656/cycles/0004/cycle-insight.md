# Cycle 0004 Insight

## Cohort outcome

Three attacks on segmentation's remaining cost were tried: a head-adjacent relink skip for protected hits (v001), a CLOCK-style second-chance reference bit replacing relink-on-get (v002), and removal of the hash avalanche fold (v003). All three were Winners. The real gains were v003's put speed and smaller struct, and v002's modest hit-rate gain with cheaper get. v001 was neutral on hit-rate and slower.

## Patterns

- The hash fold was a measurable cost on the segmented variant, mostly on put. A single multiply plus rotate gave enough mixing with no hit-rate change. This answers the cycle-0003 question: the fold, not the u32 conversion alone, was a major source of the slowdown.
- Cheaper promotion by skipping relinks only for nodes at or near the head did not pay off. Few hits land there, so the added branch cost more than it saved. Any promotion skip must cover a large share of hits, or add nothing to the hot path.
- Removing relink from get (v002) moved the work to eviction. Get got slightly faster, put tail got worse, and node size was unchanged. This is the same work-shifting pattern seen with lazy demotion in cycle 0003, though here the get hot path really did shrink.
- A single reference bit gave only a small scan-resistance gain, so it is real but limited.
- v003's struct-size drop was not targeted, which suggests the change touched more than the hash arithmetic. Attribution is inferred, since authors could not read diffs.

## Open questions

- Does the second-chance bit compose with the fold-free hasher, and do their gains add up?
- Can the eviction walk in v002 be bounded or amortized to cut the put p99 without losing the hit-rate gain?
- Why did put slow in v001, which was not its target? Possible causes are a shared touch path, code layout, or noise.
- Does the v003 struct shrink hold, and what caused it?

## Continuation

Combine v003's fold-free hasher with v002's reference-bit get path as the next baseline. Then bound the eviction scan, for example by capping the number of referenced nodes passed per put, to recover put p99. Drop head-adjacent relink-skip ideas and further hasher tweaks. Check hit-rate on workloads other than the scan-heavy trace.
