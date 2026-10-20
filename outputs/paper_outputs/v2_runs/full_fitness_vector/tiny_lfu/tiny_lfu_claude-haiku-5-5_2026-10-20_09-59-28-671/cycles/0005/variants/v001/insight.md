# Variant v001 Insight

## Helped

- per-op-time: Put latency fell at both p50 and p99 relative to the parent (cycle 0004 v002), which is the effect H01 predicted from splitting the put-side unlink/promotion path into a separate helper with a single flag read and direct relinking.

## Hurt

## Takeaway

The put-side recovery supports the parent insight's suspicion that the put regression came from the shared `unlink()`/promotion path rather than from the get-side accessor inlining. Hit-rate and metadata-size stayed at the parent's values, so the change is again a pure access-pattern edit. Two caveats: the hypothesis asked for at least three repeated runs, and the fitness vector records only one run, so the put gain is not yet separated from run-to-run noise; and the overlay source was not inspected, so it is unverified that the put-only helper exists as described. The get p99 also moved slightly even though H01 did not touch the get path, which is more likely noise than a real effect of this change.
