# Variant v002 Insight

## Helped

- per-op-time (put p99): The tail is the one place the layout change helped clearly. Put's eviction and insertion paths are where the tags matter most. They read `tags[0]` in `evict_front`, shift tags with `copy_within` on insert and remove, and rotate them in the full-bucket cold insert. With tags inline next to the `items` header, that work no longer chases a separate heap allocation, so the worst put no longer pays an extra miss for the tag buffer.

## Hurt

- per-op-time (get p50/p99): The hypothesis aimed at get and failed there. Get did not drop below the parent. It went slightly the wrong way, by an amount that is within noise. The get path really does avoid `ColdState` now. But the `#[repr(C)]` struct is still about 2 cache lines: a 24 B `items` header, then three `u32` fields, then a 64 B inline tag array starting near offset 36. So only the first ~14 tags share the first line, and a full 32-slot scan still crosses into the second. Every tag access also goes through the `tag_buf()`/`tag_buf_mut()` match on `tags_heap`, and `find_tagged` takes an extra `min` against the buffer length. Together these cancelled the line saved by dropping the separate `tags` allocation. At this cache size (2048 entries), the bucket array and tag Vecs were likely already cache-resident in the parent anyway.
- per-op-time (put p50): Every miss-put now dereferences `Box<ColdState>` for `take_ghost`, and for `record_ghost` whenever it evicts. That adds one pointer chase to the common put path, where the parent read the ghost ring from the bucket struct it had already loaded. This roughly offsets the cheaper inline tag edits at the median.

hit-rate and metadata-size held, as expected. No SLRU decision, ghost semantic, `Item` or cache header changed. metadata-size does not count the bucket struct, so the moved fields don't show up there.

## Takeaway

Moving bucket fields hot/cold and inlining tags did not lower get latency on this workload. After the tag-first scan, get is no longer dominated by bucket-struct memory touch, at least while the working set stays cache-resident. Inlining 32 tags pushes them past the first 64 B line, and the inline-vs-heap branch on every tag access eats the saving. The layout change mainly reshuffles cost on put: it removes a tag-buffer miss from the eviction tail but adds a cold-box hop to every miss. Further get gains need fewer instructions or less data movement per hit, such as the protected-segment rotate, not better struct locality.
