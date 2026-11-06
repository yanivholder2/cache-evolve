## Recurring wins

- Scan-resistant admission is the only lever that has reliably raised hit rate (cycles 1–3). It works by making new one-touch items the first evicted, either through LIP quarter-depth insert or a CLOCK bit cleared on insert. The gains are largest on wide (1024) buckets.
- The best hit-path write is a single cheap store (CLOCK reference bit). Each cycle that kept it avoided get-side regressions, and the cycle that broke it lost latency.
- Capping per-op worst-case work improves put p99. Examples are a bounded 16-slot sweep and dense absence scans.
- After cycle 1, wins have been marginal and workload-specific. Treat any new "Winner" from replacement-rule tweaks as a trade-off, not a free gain.

## Recurring failures

- Every addition to the put-miss path has raised put p50 and p99 (all three cycles): sweeps, window scans, ghost rings, dense mirrors. The quarter-depth `Vec::insert` memmove is still the dominant cost, and nothing has removed it.
- Get p50 has not moved. Reordering, scan direction, self-organizing swaps and removing dispatch overhead all left it flat or made it worse. Lookup cost tracks the number of full-key compares, not where the hit sits.
- Splitting hot data across allocations (a separate key Vec) hurts locality and makes get p50 worse.
- Fixed-size reuse signals do not generalize. BIP's blind period loses on web_0. A 16-entry ghost misses hm_0's slow-shifting reuse on small buckets. Neither wins on both workloads.
- Bounding the tail has consistently cost median latency. No change so far has improved p99 without hurting p50.
- Flat metadata-size is not evidence that a change is free. The metric cannot see per-bucket side state (key mirror, ghost ring).

## Open questions

- Does a compact inline fingerprint/tag pre-filter, kept in the same cache line, cut get p50? It is recommended every cycle and has never been tested.
- Can LIP-style insert-at-depth be approximated without a memmove (segmented or CLOCK-like), so that put latency stops growing?
- Would a reuse filter sized to bucket width (a ghost of `max_size`, or a hashed bit set) win on both hm_0 (small buckets) and web_0 (large buckets)?
- Does the hit-rate harness actually run the overlaid `evict_item` path? Cycle 1's bit-identical frequency-window result was never resolved.
- Do LIP depth and a CLOCK cleared bit combine, or are they redundant scan protection?

## Per-policy notes

- **LIP (quarter-depth insert):** a strong hit-rate baseline. Its put cost is dominated by the mid-bucket memmove. Removing dispatch overhead and reversing the scan both gave only noise-level changes.
- **BIP (bimodal MRU admission):** helps hm_0 on small buckets and gives back scan resistance on web_0. The period probably needs to scale with bucket size.
- **CLOCK / GCLOCK:** the cheapest hit path. Extra GCLOCK levels gave small but consistent hit-rate gains. Use the 16-slot sweep cap plus clear-on-insert as the base. Hit-time swaps break it (two-line writes, disturbed hand order).
- **Ghost-based admission:** a 16-entry ring keeps scans out on web_0 but does nothing on hm_0. The window has to match the workload's reuse distance.
- **Frequency-aware victim window:** still unjudged. Its cycle-1 result was identical to baseline, which is probably a harness-coverage problem. Verify that the path runs before retrying it.
