# Variant v002 Insight

## Helped

- hit-rate (large-cache cells): Both 131072 cells went up. That means a recency filter on the second hottest hit is part of what made cycle 0005 v003's strict reset work. Treating a stale second hottest hit (older than one segment's slot count of inserts) as a fresh first hit stops slow-recurring keys from taking guarded probing-bucket slots from keys that recur quickly. The gain is only about half of v003's, so the "any non-hottest hit resets the counter" signal that this variant left out carried the rest.
- hit-rate (small-cache cells): The 16384 cells did not drop, and hm_0-small went up slightly. A `PROBE_WINDOW` equal to the segment slot count is therefore not too tight for small caches. The probing guard is still on at that size, since the size gate only turns off the stash-side guard, and the window helps there too. This rules out the hypothesis's second refutation condition.
- per-op-time (get median): Get p50 did not change. That confirms the age check sits only on the rare `is_hottest` + full-probing-bucket arm. The common target-hit path does no new loads or stores, which is the key difference from v003.

## Hurt

- per-op-time (get tail): Get p99 went up. All the new work is in the hottest/full arm: loading and storing `item.timestamp`, a wrapping subtract, a compare against `PROBE_WINDOW`, and an extra branch that sometimes refreshes instead of promoting. That arm is rare enough to leave the median alone but common enough to show up in the 99th percentile. Some hits that would have promoted now return a non-promoted item and come back again later, which adds more slow-arm visits.

## Takeaway

Moving v003's recency signal into a lazy tick-age check on the guarded promotion arm keeps get p50 and the 24-byte Item. But it only recovers part of v003's large-cache hit-rate gain, and it misses the hypothesis's confirmation bar on hit-rate. This splits v003's effect into two parts: time-distance filtering accounts for some of it, and the rest needs the non-hottest-hit signal, which by design this variant does not observe. The cost moved from get p50 (v003) to get p99 (here) rather than going away. Before trying this approach again, test whether a wider or narrower `PROBE_WINDOW` changes the size of the gain. Note: this was written from the hypothesis, both fitness vectors, and the parent insight. The overlay source files could not be listed in this session, so the code-level reasons assume H02 was implemented as written. Put latency moved slightly lower even though put was not changed in this variant, so treat that as noise.
