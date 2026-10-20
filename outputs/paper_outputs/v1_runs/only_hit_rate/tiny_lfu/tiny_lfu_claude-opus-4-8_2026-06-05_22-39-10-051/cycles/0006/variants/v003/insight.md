# Variant v003 Insight

## Helped

- (none observed): The `PROMOTE_FLOOR = 2` gate on protected entry produced no movement; hit-rate is identical to the parent.

## Hurt

- (none observed): hit-rate did not regress either — the new floor branch and the extra `estimate` lookup are simply never decisive on this trace.

## Takeaway

The floor is a no-op because of ordering in `get()`: `sketch.increment(key)` runs *before* `promote()`, and `put()` already incremented the key once at admission, so by the time a probationary node reaches its first promotion its estimate is always ≥ 2 — exactly `PROMOTE_FLOOR`. The gate `promotee_freq < PROMOTE_FLOOR` can therefore never fire, and v002's promotion logic executes unchanged. To actually reserve protected slots for proven reuse on workload-a, the floor would need to be ≥ 3 (or be evaluated against a pre-increment estimate); at 2 it merely re-expresses the existing "promote on first hit" behavior.
