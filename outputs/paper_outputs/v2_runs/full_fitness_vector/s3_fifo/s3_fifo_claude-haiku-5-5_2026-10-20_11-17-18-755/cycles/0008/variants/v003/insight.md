# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Capping `freq` at 2 instead of 3 (`MAX_FREQ`) gives a hot main entry only two second-chance rotations before it becomes evictable, so entries that were recently hit are evicted slightly earlier on workload-a at 2048.
- per-op-time: The get and put bodies are unchanged apart from the constant, so the slower p50/p99 is unlikely to come from this change and should be re-measured before it is read as a cost of the ceiling.

## Takeaway

On this trace, the third frequency level in the main queue is worth keeping: dropping the ceiling to 2 costs hit rate and saves no metadata, since the index record and `Entry` sizes are identical to the parent. The change was a one-constant edit with no new work on the hit path, so any timing shift points to measurement noise rather than the policy.
