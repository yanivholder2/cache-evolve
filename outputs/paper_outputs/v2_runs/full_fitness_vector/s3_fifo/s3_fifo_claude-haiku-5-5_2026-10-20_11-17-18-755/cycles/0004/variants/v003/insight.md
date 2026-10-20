# Variant v003 Insight

## Helped

- metadata-size: Moving the per-entry frequency counter out of `Entry` into a parallel side array removed the padding that sat beside the 8-byte key and value, so `Entry<u64,u64>` is now the smaller of the two sizes the hypothesis targeted.

## Hurt

- metadata-size: The `S3Fifo` container total grew even though `Entry` shrank, because the side array for counters (and the queue-membership tag, if kept there) adds its own storage on top of the entries, so the shrink in `Entry` did not carry through to the structure as a whole.
- per-op-time: The get path now reads the counter through a second array indexed by slot, which adds an extra memory access per hit, and put/eviction paths pay the same indirection; the latency rise is in the direction the hypothesis flagged as the main risk.

## Takeaway

Hit rate is unchanged, which fits a change that was meant to be policy-preserving: the counter values at every decision point should match the parent. The change is a metadata trade rather than a win, since the saved bytes in `Entry` are more than offset in the container and the indirection costs get latency. Any future attempt at this split should measure the whole `S3Fifo` footprint, not just `Entry`. I could not read the overlay source in this session, so I have not confirmed that the implementation matches the hypothesis's invariants.
