# Variant v001 Insight

## Helped

- hit-rate (held, as required): the change only touches layout. `MAX_SLOTS` drops 64 → 16, `max_size` goes `u32` → `u8`, and the fields are reordered. Tags still only filter, every items mutation still keeps them in sync, and buckets above `MAX_SLOTS` still fall back to a plain key scan. So victim choice and recency order are bit-identical to the parent.
- metadata-size (held): `Item` and `Dash` are untouched. The only thing that shrank is the per-bucket header (about 112 B → 64 B, enforced by the new `const` assert), and that header is not part of this dimension.

## Hurt

- per-op-time (get p50/p99): the hypothesis is refuted, because `get` is flat at both percentiles. Its stated failure condition was "all get metrics flat", and that is what happened. One reason fits the parent's `#[repr(C)]` layout. Vec (0–24), `max_size`, `tag_ok` and `tags[0..16]` (about offsets 29–45) already sat in the header's first 64 B. Real buckets use the default `bucket_size` of 16, not the 2 slots the parent insight assumed, so a probe never read the second line, which held only the `[u8; 64]` tail and the `Box<dyn EvictionPolicy>`. A second factor is alignment. `DashBucket` still has 8-byte alignment (there is no `#[repr(align(64))]`), so a 64 B header in the `Vec<DashBucket>` usually straddles two cache lines anyway. "≤ 64 B" therefore does not mean "one line per header".
- per-op-time (put p99): the `put` tail regressed, and `put` p50 barely moved. Nothing on the insert path got more expensive: the `u8::try_from` runs only in `new`, and `is_full` just widens a `u8`. Moving the eviction-policy box into the first 64 B should, if anything, help eviction. So this looks like measurement variance or a layout side effect, not a cost of the change, and it should be re-measured before anyone acts on it.

## Takeaway

Packing the tagged `DashBucket` header from about 112 B to 64 B did not help, because the parent's hot probe fields (items Vec, `len`, the first 16 tag bytes) already shared one cache line. The extra line held only cold padding and the eviction-policy box, which is touched only on a hit or an eviction. So the parent's "lower bound" caveat was wrong: inline tags already delivered their full gain in cycle 9. Further `get` wins will not come from header size alone. If layout is revisited, the untested lever is real 64 B alignment of the bucket array, not a smaller struct.
