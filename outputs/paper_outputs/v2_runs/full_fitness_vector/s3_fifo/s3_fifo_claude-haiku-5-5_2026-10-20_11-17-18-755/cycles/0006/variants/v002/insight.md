# Variant v002 Insight

## Helped

- metadata-size: Merging the two per-queue `HashMap` indexes into one id-keyed map removes a whole map's worth of container state from `S3Fifo`, so the container is smaller while `Entry` is unchanged.
- put: A single map means a push, eviction, or promotion touches one index instead of one of two, and the resident-key check no longer has to consult two maps, so both put p50 and p99 come down.

## Hurt

- get: The one lookup now returns a `Slot` that carries the queue tag and stored position, so the hit path pays a small decode step to pick the queue before its indexed access; the cost is within noise.

## Takeaway

Collapsing the indexes into one tagged map trades a fixed container saving for a small per-get decode, and hit rate is unchanged because eviction, ghost, and promotion decisions were left alone. The change is a structural win on size and put cost, with get essentially flat rather than improved.
