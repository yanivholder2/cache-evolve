# Variant v003 Insight

## Helped

_(no dimension improved relative to v002)_

## Hurt

- **per-op-time**: The `get` p50 regressed from the timer-floor reading v002 produced to a full 100 ns tick; `#[inline(always)]` on `unlink` and `push_back` expanded `get`'s compiled footprint, and the larger inlined body appears to have pushed the hot path out of optimal instruction-cache placement rather than reducing call overhead.
- **per-op-time**: The `put` p99 widened; the eviction (full-cache) branch — the dominant steady-state path — gained an extra `VacantEntry::into_key()` step to satisfy the borrow checker before `map.remove()` could be called, and the instruction-cache pressure from the inlining changes likely added to tail-latency variance on that path.

## Takeaway

The `entry()` API can only collapse two hash probes into one on the not-full (warmup) path; once the cache reaches capacity, the borrow-checker constraint forces the VacantEntry to be released before `map.remove()` runs, so the eviction path still pays the same number of probes as v002 plus an extra pointer move. Combined with the code-size increase from `#[inline(always)]` on the two hot helpers, the change produced the opposite of its intent: it raised `per-op-time` on both the get and put p99 without touching hit-rate or metadata footprint.
