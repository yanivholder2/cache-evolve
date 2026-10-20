# Variant v001 Insight

## Helped

## Hurt

- per-op-time (put): Restricting the full-bucket victim scan to occupied, unreferenced slots adds a mask AND and a fallback branch to every full-bucket put, and the put median and tail both rose instead of falling, so the added per-put work outweighed any shorter scan on this trace.
- per-op-time (get): The hypothesis leaves the get path untouched, so the slower get median and tail are not explained by the victim-selection change and most likely reflect run-to-run variance or code-layout shifts.

## Takeaway

Narrowing victim selection to unreferenced slots did not shorten the put tail, which is the one thing H01 targeted; hit-rate and metadata-size stayed at the parent's values, consistent with the no-new-state invariant. The put regression suggests the full-bucket scan length was not the dominant tail cost at this size, so the next put-side profile should look elsewhere. The overlay source could not be read in this session, so whether the fallback fires often on this trace is unconfirmed.
