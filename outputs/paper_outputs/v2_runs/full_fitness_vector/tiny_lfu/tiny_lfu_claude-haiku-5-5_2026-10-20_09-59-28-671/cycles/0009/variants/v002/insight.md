# Variant v002 Insight

## Helped

## Hurt

- per-op-time: The get path, which this variant targeted by inlining the packed protected-flag accessor calls in `touch()` and the get-reached promotion step, timed slower than the parent, and the put path (which the hypothesis said should be left as in the parent) also timed slower, so the regression is not confined to the edited get sites.

## Takeaway

Inlining the get-side flag accessors on top of the put-split parent did not produce the get-side gain H02 predicted; both get and put timings rose together in a single run. Because the hypothesis kept put-reachable call sites unchanged, a put regression is unexpected from this edit alone, which points to either measurement noise (the fitness vector records one run, and the hypothesis asked for at least three with spread reported) or an overlay that differs from the described change. The overlay source could not be inspected in this session, so the code-level cause is unverified; hit-rate and metadata-size stayed at the parent's values, consistent with a pure access-pattern edit.
