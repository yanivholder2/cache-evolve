# Variant v001 Insight

Note: I could not list or diff the overlay source in this session (Bash and Glob were unavailable). This insight rests on the hypothesis and the two fitness vectors, not on a line-level diff.

## Helped

- hit-rate: No movement. The tag-first scan keeps the same match semantics, admission filter and eviction order, so hit-rate is unchanged, as the hypothesis required.
- metadata-size: No movement. Item and Dash sizes are unchanged, so the tags fit in spare header bits with no new fields.

## Hurt

- per-op-time: Both get and put got slower than the parent, and the tails widened too. The tag-first scan was meant to cut get time, but it added work to the hot path. Likely sources are packing, extracting or masking tags and a second pass over the slots. On a small bucket, the parent's direct key compare was already cheap. Put also got slower even though the hypothesis targeted get, so the put existing-key check or tag maintenance on insert probably adds cost. I can't confirm either without the source diff.

## Takeaway

Replacing the bucket-slot scan with a tag-first comparison made latency worse, not better. That suggests the bucket scan is not where hit-path time goes. For u64 keys, a full-key compare is as cheap as a tag compare, and tag bookkeeping adds overhead. Future latency work should look outside the slot scan, for example at locking, hashing or memory layout and cache misses.
