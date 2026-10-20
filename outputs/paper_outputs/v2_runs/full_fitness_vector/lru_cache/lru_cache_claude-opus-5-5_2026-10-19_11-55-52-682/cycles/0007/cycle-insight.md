# Cycle 0007 Insight

## Cohort outcome

This cycle tried one representation change and two ghost-policy changes. v001 moved VISITED from cycle 0006 v002's `links` word into a dense bitset. v002 placed ghost returners at the SIEVE hand. v003 halved the direct-mapped ghost. All three are listed as Winners, but only v001 is a clear win: `get` and the `put` tail are faster and eviction order is unchanged. v003 is a near-noise hit-rate gain that comes with a free heap saving. v002 lost hit-rate.

## Patterns

- Both ghost probes this cycle point the same way: limit how far back the ghost reaches, but don't weaken what happens after a ghost hit. Returners placed at the hand were evicted before they could reuse. Shortening the ghost's reach cost nothing and may have helped slightly. A ghost hit is a strong reuse signal. Recall is the knob worth tuning, not admission placement.
- Flags that get written on every hit belong in a tiny, always-cached structure. The bitset recovered most of the side array's `get` penalty and made hand sweeps cheaper at the `put` tail. Each extra side array still adds fixed work on the miss path and grows the struct, so the median `put` rose slightly.
- S3-FIFO knobs on workload-a are now a flat plateau. A fourth straight cycle of admission and ghost tweaks produced only noise-sized moves in either direction.
- Latency attribution is getting unreliable. v002 slowed on a `get` path its change never touched, so environment drift is now as large as the effects being measured.
- Metadata accounting still misses heap side structures. v003's real ghost-buffer saving and v001's bitset don't show up in it.

## Open questions

- Should main evictions be recorded in the ghost? v002 found that returners evicted from main lose their fast path.
- Where does hit-rate start to fall as the ghost shrinks further, and does that point move with cache size?
- Can a single fused node-plus-flag layout give the bitset's `get` gain without an extra array header or extra work on every miss?
- Does any of the S3-FIFO lineage hold at another cache size or trace? This has gone unverified for six cycles.

## Continuation

Use v001's bitset layout combined with v003's half-size ghost as the new parent. Both keep eviction order safe, and they should stack. Drop placing returners at the hand, and stop sweeping small-queue or ghost size on workload-a. Before more policy tuning, add a second cache size or trace and repeat runs so latency deltas can be told apart from drift. The one policy probe still worth trying is recording main evictions in the ghost.
