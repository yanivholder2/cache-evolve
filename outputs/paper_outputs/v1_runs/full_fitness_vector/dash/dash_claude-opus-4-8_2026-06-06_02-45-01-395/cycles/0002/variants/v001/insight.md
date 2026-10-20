# Variant v001 Insight

## Helped

- metadata-size: Dropping the `timestamp: Instant` field and narrowing `lfu_counter` from `usize` to `u8` shrinks `Item<u64,u64>` by removing two words of dead per-entry footprint.
- per-op-time: A smaller `Item` makes every `Vec::remove`/`push` move and `Clone` in the hot `get`/`put` paths copy fewer bytes, trimming both get and put p50/p99 on top of the inherited MRU-promotion win.

## Hurt

- hit-rate: Unchanged — the slimming only touched `Item` layout, and the load-bearing `InsertPosition::Back` promotion in `get` was kept verbatim, so placement behaviour (and thus hits) is identical.

## Takeaway

`Item`'s `timestamp` was pure footprint: no policy on this trace read it, so deleting it and shrinking the LFU counter compounds with the parent's promotion winner — same hit-rate, smaller entries, faster ops. Placement and footprint are genuinely orthogonal axes here, and porting v003's slim `Item` onto v001's `get` adds their wins rather than trading one for the other.
