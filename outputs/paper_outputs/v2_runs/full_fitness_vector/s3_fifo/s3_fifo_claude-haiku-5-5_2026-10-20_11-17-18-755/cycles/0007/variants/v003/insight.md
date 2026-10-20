# Variant v003 Insight

## Helped

- per-op-time: The single `index: HashMap<K, Slot>` replaces the two linear `position` scans in `get()` with one hash lookup plus a direct indexed access into `small`/`main`, which is the likely source of the get-side drop.
- per-op-time: On a resident key, `put()` now finds the slot through the index instead of scanning both deques, so the update path no longer pays for a linear walk; the put-miss path still uses the fingerprint ghost's hash-set check inherited from the parent, which is where the remaining put gain from the parent is retained.

## Hurt

- metadata-size: The `S3Fifo` container gained the index map and the two head counters (`small_head`, `main_head`), a fixed cost on top of the parent's ghost structures; `Entry` is unchanged, so the increase is entirely in the container.

## Takeaway

The index and the fingerprint ghost touch disjoint paths (index on resident get/put, ghost on the put-miss path), so their gains stack without interfering and hit rate is unchanged as the hypothesis predicted. The index is paid for in container size, since every resident key now also lives in a hash map, so any later comparison should weigh the `S3Fifo` footprint rather than `Entry` alone. This single variant cannot separate how much of the put-side improvement comes from the index versus the inherited ghost change; that would need an ablation with the index removed.
