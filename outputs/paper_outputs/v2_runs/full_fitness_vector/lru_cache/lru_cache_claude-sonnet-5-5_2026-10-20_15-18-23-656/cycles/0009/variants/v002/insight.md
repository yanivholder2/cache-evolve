# Variant v002 Insight

## Helped

(none — no dimension improved over the parent)

## Hurt

- per-op-time: Both get and put got slower at p50 and p99, so the in-place victim-slot reuse in `put` did not cut the put cost the hypothesis targeted. The extra overwrite and relink work, and possibly changed code layout or inlining, appear to cost more than the avoided allocate/free pair. `get` was not modified, so its slowdown points to layout or measurement noise rather than the logic change.

## Takeaway

Recycling the evicted node's slot saved nothing measurable here. The allocator and free pair was evidently not the dominant cost of a put, since the map remove and insert still run for the victim and new keys. The `get` regression, with `get` untouched, suggests that code-layout or measurement effects can outweigh the small structural gains of this kind. Future put-path attempts should be confirmed against run-to-run noise on `get` before being credited or blamed. Hit-rate and metadata-size were unchanged, which confirms the policy and node layout were preserved.

Caveat: I could not read the source overlays in this session, so the code-level explanation is inferred from the hypothesis and the fitness vectors.
