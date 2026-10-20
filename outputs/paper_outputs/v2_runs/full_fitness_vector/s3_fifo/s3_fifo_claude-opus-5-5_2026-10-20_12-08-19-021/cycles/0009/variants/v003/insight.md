# Variant v003 Insight

## Helped

- per-op-time (`get`, the summary): `get` is byte-for-byte the same as the parent's: one `index.get_mut`, a packed freq bump, and one slab load. The small p50 and p99 movement therefore can't be credited to H03. It is most likely benchmark noise, or a side effect of a different resident set under the new policy. It should not be read as a gain from this change.

## Hurt

- hit-rate: The ghost-hit branch still moves one slot from small to main on every ghost hit, but it no longer puts the hit key into main. That breaks the link between "main grows" and "main gets filled". Small is pushed toward its `capacity / 64` floor. The re-admitted key is pushed into that shrinking small with freq 1. A ghost key was evicted from small precisely because its reuse distance was longer than small, so it is very unlikely to get a second `get` inside an even smaller window. It is evicted from small, ghosted again, and its next re-put is another ghost hit that shrinks small further. The result is a self-reinforcing loop. Long-reuse keys bounce between small and ghost and never become resident long enough to hit. The extra main capacity can only be filled by graduates from a starved small queue, so a sizeable part of the cache likely holds no useful keys.
- per-op-time (`put`): The ghost-hit path used to do at most one `evict_main` before pushing to main. It now ends with a full `evict_small`. That call can graduate a victim, which may cascade into `evict_main` and the grow-small fix-up, or it can release the victim and `ghost_push` a new fingerprint. Combined with the extra ghost churn from the bounce loop above, this makes each `put` do more work at both the median and the tail.
- metadata-size: Did not change. `alloc_with_key_freq` only writes the freq bits as part of the existing `index.insert`, and the struct and `Entry` layouts are untouched.

## Takeaway

In this adaptive S3-FIFO, sending a ghost hit straight to main is not a leak to be plugged. It is what makes the ghost-driven split adaptation pay off, because the slot that main gains on a ghost hit is immediately used by the key that proved main was too small. If you keep the adaptation but send the key back through small, two things break together: main grows with nothing to fill it, and small shrinks until confirming reuse becomes impossible. This is the falsifying outcome the hypothesis predicted, so direct readmission should be kept. Any future attempt to weaken readmission has to weaken the small-shrink signal along with it. The rationale's claim that long-reuse keys displace main residents is also not supported here, since keeping them out of main cost far more than it saved. I couldn't check the `settled_small` and `floor_blocks` values that `h01_split_report` was meant to print against the eval artifacts. Seeing small pinned at the floor would confirm the mechanism described here.
