# Variant v002 Insight

## Helped

- hit-rate: Stamping each ghost fingerprint with a push sequence and counting a ghost hit only while that stamp is inside the last `ghost_cap` pushes lets the ghost hold roughly half the main queue's worth of recent evictions, so more re-referenced keys are recognised than under the parent's eager drain.
- per-op-time: Replacing the eager ghost removal with a batched `retain` sweep that runs about once every `ghost_cap` pushes moves expiry off the per-eviction path, which is the likely reason put-side latency improved relative to the parent.
- metadata-size: Collapsing the parent's three ghost structures (queue, set, and copy-count map) into one `HashMap<u64, u64>` plus a single `ghost_seq` counter removes the duplicated fingerprint storage, so the `S3Fifo` container footprint shrank even though `Entry` is unchanged.

## Hurt

## Takeaway

The gain here comes from changing how the ghost expires, not from how much it stores: a push-count window with lazy batched pruning recovers the hit rate that the strict cap gave up, while keeping the put path cheaper. The hit-rate gain is small, so the window approximation is close to the strict cap but not identical; the ghost population this variant actually holds should be checked against the parent's before the window size is tuned further.
