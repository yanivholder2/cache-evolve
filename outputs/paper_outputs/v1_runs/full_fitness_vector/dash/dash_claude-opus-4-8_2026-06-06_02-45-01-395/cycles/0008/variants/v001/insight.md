# Variant v001 Insight

## Helped

- (none) — no dimension improved; the reorder's intended get-p50 win did not materialize.

## Hurt

- per-op-time: Moving the stash probe last regressed get-p50 — the median get hit actually resolves in the stash, so deferring that branch forced the dominant path to pay two failed `get_position` scans (target + probing) before reaching the hit it used to find first.

## Takeaway

The hypothesis assumed the hottest hits live in the target/probing tiers and waste a stash scan; the measurement says the opposite — on this LIRS/Zipfian trace the *median* get resolves in the stash, so probing target → probing → stash adds two negative scans to the common path instead of removing one. hit-rate and metadata stayed bit-for-bit identical as predicted, confirming the reorder was behavior-preserving; it was the cost model that was inverted. The branch-ordering lever exists, but it points stash-first, not stash-last, for get-p50 on this workload.
