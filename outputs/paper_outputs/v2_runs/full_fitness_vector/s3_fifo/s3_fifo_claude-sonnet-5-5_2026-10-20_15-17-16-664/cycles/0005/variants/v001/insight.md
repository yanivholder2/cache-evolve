# Variant v001 Insight

## Helped

- per-op-time (put): Replacing the ghost trimming with a fixed-capacity ring buffer that does a single pop_front makes trimming constant-time. Both the median and the tail of `put` dropped, and the tail drop confirms the parent's finding that eviction and ghost trimming drove the tail.
- hit-rate: Admission, promotion and eviction order were held fixed, so the hit/miss sequence is unchanged. This confirms the ghost container change is behavior-neutral.

## Hurt

- per-op-time (get): Both the median and the tail of `get` got slower, and the per-op-time summary tracks `get` p50, so the headline got worse even though `put` improved. The change did not touch the `get` path, so the likely causes are layout or cache-footprint shifts or measurement noise. I could only read the fitness vectors, not the source, so this is unconfirmed.
- metadata-size: The per-entry size summary is unchanged, but the whole-cache struct grew slightly. The ring buffer's head/length bookkeeping is the plausible source. The summary metric does not show it.

## Takeaway

Making ghost trimming O(1) confirmed that the `put` tail comes from ghost maintenance, and it did so without changing hit-rate. The per-op-time summary reads only `get` p50, which is why the headline regressed while `put` improved. The next variant should treat `get` p50 as a separate target and check whether the `get` slowdown is real or noise.
