# Variant v001 Insight

## Helped

- hit-rate: Keeping the deferred stash-to-target promotion from the parent while moving its first-hit state into existing stash slot metadata preserves the parent's reason for gaining hit rate, and the small gain holds on workload-a.
- per-op-time: Removing the separate load and conditional update from the stash-hit branch recovered most of the get tail that the parent's first-hit check had added, which is the stated purpose of the change.

## Hurt

- per-op-time: The get median moved up slightly, which fits the new flag-bit handling still adding a little work on every stash hit, though the difference is small enough that it may be noise.

## Takeaway

This variant tests whether the parent's get-tail cost came from the first-hit state itself rather than from the deferred promotion, and the result points at the state: the tail recovered while the hit-rate gain was kept. The put-side tail also improved sharply even though the hypothesis required the put path to be unchanged, so that movement is not explained by this edit and should not be credited to it until it is reproduced. All evidence comes from a single trace (workload-a at size 2048), which does not include the stash-dominant workload the parent's insight asked for, so the result is not yet a test of the deferral benefit.
