# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-29T10:04:23+00:00

## timeline

### 10:04:38 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Scan-resistant admission is the only lever that has reliably raised hit rate (cycles 1–3). It works by making new one-touch items the first evicted, either through LIP quarter-depth insert or a CLOCK bit cleared on insert. The gains are largest on wide (1024) buckets.\n- The best hit-path write is a single cheap store (CLOCK reference bit). Each cycle that kept it avoided get-side regressions, and the cycle that broke it lost latency.\n- Capping per-op worst-case work improves put p99. Examples are a bounded 16-slot sweep and dense absence scans.\n- After cycle 1, wins have been marginal and workload-specific. Treat any new \"Winner\" from replacement-rule tweaks as a trade-off, not a free gain.\n\n## Recurring failures\n\n- Every addition to the put-miss path has raised put p50 and p99 (all three cycles): sweeps, window scans, ghost rings, dense mirrors. The quarter-depth `Vec::insert` memmove is still the dominant cost, and nothing has removed it.\n- Get p50 has not moved. Reordering, scan direction, self-organizing swaps and removing dispatch overhead all left it flat or made it worse. Lookup cost tracks the number of full-key compares, not where the hit sits.\n- Splitting hot data across allocations (a separate key Vec) hurts locality and makes get p50 worse.\n- Fixed-size reuse signals do not generalize. BIP's blind period loses on web_0. A 16-entry ghost misses hm_0's slow-shifting reuse on small buckets. Neither wins on both workloads.\n- Bounding the tail has consistently cost median latency. No change so far has improved p99 without hurting p50.\n- Flat metadata-size is not evidence that a change is free. The metric cannot see per-bucket side state (key mirror, ghost ring).\n\n## Open questions\n\n- Does a compact inline fingerprint/tag pre-filter, kept in the same cache line, cut get p50? It is recommended every cycle and has never been tested.\n- Can LIP-style insert-at-depth be approximated without a memmove (segmented or CLOCK-like), so that put latency stops growing?\n- Would a reuse filter sized to bucket width (a ghost of `max_size`, or a hashed bit set) win on both hm_0 (small buckets) and web_0 (large buckets)?\n- Does the hit-rate harness actually run the overlaid `evict_item` path? Cycle 1's bit-identical frequency-window result was never resolved.\n- Do LIP depth and a CLOCK cleared bit combine, or are they redundant scan protection?\n\n## Per-policy notes\n\n- **LIP (quarter-depth insert):** a strong hit-rate baseline. Its put cost is dominated by the mid-bucket memmove. Removing dispatch overhead and reversing the scan both gave only noise-level changes.\n- **BIP (bimodal MRU admission):** helps hm_0 on small buckets and gives back scan resistance on web_0. The period probably needs to scale with bucket size.\n- **CLOCK / GCLOCK:** the cheapest hit path. Extra GCLOCK levels gave small but consistent hit-rate gains. Use the 16-slot sweep cap plus clear-on-insert as the base. Hit-time swaps break it (two-line writes, disturbed hand order).\n- **Ghost-based admission:** a 16-entry ring keeps scans out on web_0 but does nothing on hm_0. The window has to match the workload's reuse distance.\n- **Frequency-aware victim window:** still unjudged. Its cycle-1 result was identical to baseline, which is probably a harness-coverage problem. Verify that the path runs before retrying it.\n"
}
```

### 10:04:44 chat

I wrote `cross-cycle-insights.md` in `cycles/0003/cross-cycle/`. It has the four required sections in order, built only from the ledger for cycles 1–3.

- **Recurring wins:** Making new one-touch items the first to be evicted (scan resistance) is the only change that has reliably raised hit rate. Recording hits with a single cheap bit store protects get latency, and capping worst-case work per operation improves put p99. Since cycle 1, wins have been small and tied to specific workloads.
- **Recurring failures:** Every extra step on the put-miss path has made put latency worse. The memmove from inserting at quarter depth is still the main cost. Get p50 has not responded to reordering or scan direction, because it tracks how many full-key compares are done. Keeping keys in a separate array hurts locality. Fixed-size reuse signals don't work on both workloads. The metadata-size metric can't see state held inside each bucket.
- **Open questions:** The inline fingerprint/tag pre-filter has been recommended every cycle but never tried. The other open threads are:
  - inserting at depth without the memmove;
  - a reuse filter sized to the bucket;
  - whether the hit-rate harness runs the overlaid eviction path at all, still unresolved since cycle 1;
  - whether LIP depth and a CLOCK cleared bit overlap.
- **Per-policy notes:** separate entries for LIP, BIP, CLOCK/GCLOCK, the ghost ring, and the frequency-aware victim window, which is still unjudged because of the harness question.


---

- finished: 2026-10-29T10:04:45+00:00
- duration: 21.68s
- status: ok
- input_tokens: 4
- output_tokens: 1925
- cache_read_tokens: 37163
- cache_write_tokens: 8630
- cost_usd: 0.114989
