# Variant v003 Insight

## Helped

- per-op-time: Put latency fell at both median and tail because the ghost queue now holds 64-bit fingerprints computed once with the cache's existing hasher, so eviction no longer clones and stores full keys on the put miss path.

## Hurt

- metadata-size: The `S3Fifo` struct grew because the fingerprint ghost adds a `HashMap<u64, u32>` count index alongside the `VecDeque<u64>`, so the fixed-size `Entry` win is offset by heavier cache-level bookkeeping.

## Takeaway

Replacing ghost keys with fingerprints is a real put-path win, and it came from dropping key clones, not from the O(1) index the hypothesis emphasized. The implementation departs from the hypothesis in ways that matter: it uses a growable deque and a count-based index instead of a fixed ring and set, and a ghost hit does a linear `retain` over the ghost queue, a tail-latency risk that did not show up at this capacity. Hit rate did not move on this trace, so fingerprint collisions were not measurable here, and the get-path change is not attributable to this edit because the get code was not touched.
