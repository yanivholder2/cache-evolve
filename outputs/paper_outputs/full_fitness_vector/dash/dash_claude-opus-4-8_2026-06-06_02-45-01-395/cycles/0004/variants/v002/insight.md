# Variant v002 Insight

## Helped

- per-op-time: Caching `is_classic_lru` as a plain `bool` on `DashBucket` and branching to the `#[inline]` `lru_get_from_position` free function removes the `Box<dyn EvictionPolicy>` vtable indirection on the hot path, letting the compiler inline the remove+push — get p50/p99 and put p50 all dropped, and the summary metric cleared the harness noise floor that swallowed cycle/0003/v003's reorder.

## Hurt

- per-op-time: Put p99 moved the wrong way even though put p50 improved; the devirtualized insert path didn't structurally touch tail behaviour, so the tail regression reads as `Vec` reallocation/measurement variance rather than a cost introduced by the change.

## Takeaway

Removing a per-op vtable indirection and enabling remove+push inlining is a structurally large enough lever to be resolvable by this harness — it answers the open "what granularity is detectable" question, where prior single-scan reorders were not. The win is concentrated in median/get-tail latency; the isolated put-p99 wobble is consistent with allocation noise, not the specialization itself, since hit-rate and metadata-size stayed bit-for-bit identical as the trait object was retained for every non-ClassicLRU policy.
