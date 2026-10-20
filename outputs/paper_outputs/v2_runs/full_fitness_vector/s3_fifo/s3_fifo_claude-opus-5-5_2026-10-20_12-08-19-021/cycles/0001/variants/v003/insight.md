# Variant v003 Insight

## Helped

- hit-rate: Letting the small/main split move by ±1 slot (ghost hit → main grows, graduate-into-full-main → small grows) beat the baseline's fixed 25/75 split on workload-a. The per-key movement rules were left alone, so the gain comes only from how much room each queue gets. It shows the 25/75 split is not the best split for this trace.
- per-op-time: `get` and `put` latencies both went down at p50 and p99, but the diff doesn't explain why. It adds branches and extra fix-up evictions, and leaves the O(capacity) linear scans in `get`/`put` and the ghost `iter().any` + `retain` as they were. The most this change could plausibly contribute is fewer full-length miss scans from the higher hit-rate. Treat this gain as unexplained and re-measure it before crediting it to H03.

## Hurt

(none — `metadata-size` didn't move: `Entry<K,V>` is unchanged. Adding the `capacity` and `ghost_cap` fields grew `S3Fifo` by two words, but that is a fixed per-cache cost and not part of the per-entry summary.)

## Takeaway

ARC-style feedback on queue sizes improves S3-FIFO on workload-a even when admission and promotion rules are unchanged. That makes it a separate lever that can be stacked with the H01/H02 movement changes. The two triggers are lopsided, though. "Main is full when a key graduates" is true almost all the time in steady state, so the split ends up as a tug-of-war between how often ghost hits happen and how often keys graduate, held inside the [capacity/16, capacity/2] limits. Later variants should log where `small_size` settles, to tell whether the gain comes from the adaptation itself or just from landing on a better fixed split.
