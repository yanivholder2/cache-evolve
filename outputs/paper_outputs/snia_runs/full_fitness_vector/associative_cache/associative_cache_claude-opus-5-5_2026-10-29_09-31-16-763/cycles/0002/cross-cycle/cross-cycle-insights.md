## Recurring wins

- Scan-resistant admission is the only lever that has won on hit rate twice. Both Winners (cycle 0001 LIP and CLOCK, cycle 0002 BIP-on-LIP) make new items earn protection before they are shielded from eviction. The gains are largest on the wide buckets.
- Bounding worst-case work helps the put tail every time. Examples so far are the 16-slot sweep cap, clear-on-insert and the dense absence scan.
- Metadata-size has stayed flat. Reusing existing per-item fields such as `lfu_counter` and changing only the insertion position cost nothing on that axis. Note that this metric cannot see side arrays held inside the bucket.

## Recurring failures

- Every change to the eviction or insert mechanism has made the median put slower, in both cycles. Mid-bucket memmoves, per-item sweeps, window scans and writes to two allocations all cost more per op than they save. "A memmove is a memmove" did not hold.
- Changing the hit path does not move get p50. It barely moved even when the hit path became a single store. The linear key lookup across the wide bucket dominates the get cost.
- Splitting hot data across allocations makes get p50 worse. The separate dense key Vec added a second memory stream to every hit.
- Trimming policy dispatch (the per-op `name()` string compare) gave no visible speed gain. The mechanism cost hides it.
- Returns from tweaking the replacement rule are shrinking. After cycle 0001 the gains are small, and each one trades hit rate between workloads (BIP gave back scan resistance on web_0).

## Open questions

- Does the hit-rate harness run the overlaid bucket `evict_item` / ClassicLRU paths? The frequency-window variant matched the baseline exactly and has not been re-checked.
- Can inline per-slot fingerprints or tags in the same cache line, or SIMD-friendly blocks, cut get p50 where a separate key Vec did not?
- Is there a cheaper insert-at-depth for LIP-style policies that avoids the mid-bucket memmove?
- Should the insertion depth or BIP period scale with bucket width, or use a reuse-aware admission signal, to keep the small-bucket gains without losing web_0?
- Do LIP-style depth insertion and a cleared CLOCK/GCLOCK reference bit add up, or do they give the same scan protection twice?
- Can any change to bound the tail avoid adding cost at p50? So far, tail and median have moved in opposite directions.

## Per-policy notes

- LIP / quarter-depth insert: a Winner in cycle 0001 and the base of the cycle 0002 Winner. Its hit rate is strong, but its put latency is consistently poor because of the mid-bucket memmove.
- BIP (bimodal on LIP): only a marginal Winner, carried by one small-bucket cell. It loses on web_0 and does not fix put latency.
- CLOCK / GCLOCK: a Winner in cycle 0001. Going to 2 bits added only a little hit rate, though it did so at every point. With the bounded 16-slot sweep and clear-on-insert it is the preferred base mechanism for the put tail, although it makes the typical put slower. Unbounded sweeps hurt the tail.
- Frequency-aware tail window: has no measurable effect yet, and the harness coverage is unconfirmed. Do not judge it until that is checked.
- Lookup layout (dense key mirror): improved the put tail but hurt get p50 through poor locality. Drop the separate-Vec approach.
