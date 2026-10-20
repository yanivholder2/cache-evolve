# Variant v001 Insight

## Helped

- hit-rate: Capping the fingerprint ghost at half the main-queue capacity, with the drain done on the ghost-push path, appears to keep the ghost to the most recent evictions, so the ghost-hit promotion fires on fresher evidence than v003's unbounded fingerprint ghost.

## Hurt

- per-op-time: The drain runs inside every small-queue eviction, so each put now also pays for removing the drained fingerprints from `ghost_set` and updating `ghost_copies`; that bookkeeping sits directly on the put path and explains the put-side regression.
- per-op-time: The get path is unchanged by the diff, so the get-side regression is not explained by the ghost cap and should be treated as noise or a code-layout effect rather than a cost of this change.
- metadata-size: The `ghost_cap` word adds one `usize` to the `S3Fifo` container, which is the only footprint change the diff makes; `Entry` is unchanged at 16 bytes.

## Takeaway

The half-main ghost bound does carry over onto the fingerprint ghost, giving a small hit-rate gain, but it is paid for on the put path by the per-eviction drain bookkeeping. Any further work on this line should either make the drain amortized (for example, compact in batches rather than per push) or show the put-side cost is avoidable before counting the hit-rate gain as a net win.
