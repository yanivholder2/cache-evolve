# Variant v002 Insight

## Helped

- per-op-time (get tail): Get p99 fell back to about where it was before v003's guard, as H02 predicted. The guarded promotion into a full probing bucket no longer runs the probing-LRU → foreign-retry → target-LRU → stash cascade. It now does a two-bucket swap: the hot key goes into probing and the probing LRU entry goes into the freed target slot. The slowest path behind the tail is gone, not just deferred. Get p50 did not move, which fits a change that only touches a rare arm.

## Hurt

- per-op-time (put tail): Put p99 rose, by about as much as get p99 fell. `put` was not edited. One possible indirect cause: the swap leaves the demoted entry at target's `Back` instead of sending overflow to foreign/stash. Targets then stay fuller, and later puts more often hit their own eviction/displacement path. This is not confirmed, and it may be noise. v003's put tail already moved by a similar amount with no edit to `put`.

## Takeaway

Replacing the cascade with a direct swap removed the guard's get-tail cost and left hit-rate exactly the same in all four trace/size cells. That fact is the main lesson. Where the cascade sent the displaced probing LRU entry (foreign bucket, target-LRU, stash) made no measurable difference to which keys stayed cached. The cascade was pure latency cost. The hit-rate gain comes entirely from v003's guard, i.e. *when* promotion happens, not from *how* the full-bucket move is done. Because hit-rate matches to the last digit, this guarded full-probing arm may fire rarely on these traces. If so, the remaining get/put tails probably come from other paths, such as stash→target or put eviction, not from this arm. This insight was written from the hypothesis's change description and the fitness vectors. The overlay source could not be found with the available tools, so the code itself was not checked.
