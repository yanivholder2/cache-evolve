# Variant v002 Insight

## Helped

- hit-rate: Slightly higher than the parent even though the hypothesis changed no policy, insertion index or hashing. Alignment alone shouldn't change hit rate, so this is probably a side effect of the layout change rather than a real gain.

## Hurt

- per-op-time: Both get and put latency got worse than the parent, and put p50 and p99 got much worse. Forcing 64-byte alignment on the bucket storage did not cut put time. The likely cause is extra padding and a larger footprint that adds memory traffic and works against the parent's swap-insertion path.

## Takeaway

Plain `#[repr(align(64))]` on the bucket storage, stacked on swap insertion, does not speed up puts for this 40-byte `Item<u64,u64>` layout. It left metadata-size unchanged but made latency worse, so cache-line alignment is not a free win here. Any further layout work needs to show a measurable reduction in cache-line touches per put before it is worth keeping.

(Caveat: I could not read the overlay source in this session, so the causal explanation comes from the hypothesis text and the fitness vectors, not from the code diff.)
