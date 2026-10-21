# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:45:57+00:00

## timeline

### 09:46:13 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Estimate quality was the only large hit-rate gain in eight cycles. Widening the count-min sketch (cycle 0003) removed collision overcount. Cheap or hash-only changes (fast single hash, cached-hash fast path) kept hit-rate and promised lower cost.\n- Making counts honest helped a little. Deduplicating the get-miss-then-put double increment (cycle 0004) let true access counts decide the contest.\n- Lifetime, unaged frequency is the asset on workload-a. Keeping strict `>` with keep-victim-on-tie, and protecting long-reuse-distance keys, has held up in every test against it.\n- Policy changes now win only by holding hit-rate, or by losing the least. From cycle 0005 on, Winners mostly tied the parent (SLRU split, frequency-based demotion, ghost tie-break, ghost margin).\n\n## Recurring failures\n\n- Aging or decay, tested five ways: aggressive halving, long-period halving, halving on a clean sketch, global halving on SLRU+dedup, and per-entry idle discount. Every variant lost or stayed flat, and later clean tests (cycles 0004, 0008) ruled out the earlier confounders.\n- Contest tweaks: tie-admission, ghost tie-breaks, ghost-hit margins and random admission jitter. Exact ties at warm estimates are rare. When the contest is loosened, cold probation keys just churn, and higher-count keys get swapped for lower-count ones.\n- Victim sampling depth. Wider or deeper samples evict young, still-useful keys, because without aging the lowest estimate marks the newest resident.\n- Resident-structure levers: a 1% W-TinyLFU window (tested on both noisy and clean estimates), a hill-climbing adaptive window, SLRU ratio sweeps, and idle-based protected demotion. The min-estimate victim sample already acts as a soft protected segment, so these rearrange entries without changing which keys get evicted.\n- More sketch-accuracy work after the wide sketch: conservative update, more width, more aging. Width and conservative update fix the same error, so their gains don't add up.\n- Bundling two mechanisms in one variant (cycles 0001, 0008) made the result hard to attribute.\n\n## Open questions\n\n- Per-op-time and metadata-size have not been scored in any of the eight cycles. Which lineage wins once they are? Candidates are the fast hash, the cached-hash fast path with ≤1 shortcuts, the narrow u16 conservative-update sketch, the SLRU per-hit cost, and the ghost-ring overhead.\n- How narrow can the sketch get? Width 8× was neutral without SLRU but slightly negative with it. Would a recency tie-break between equal estimates let it go narrower?\n- Two questions remain only to close threads: very long-period halving, and raw-estimate protected-head demotion without a discount. Both are expected to come out neutral or negative.\n- Does the workload-a plateau carry over to other workloads, where recency, a window or aging might pay off?\n\n## Per-policy notes\n\n- TinyLFU sketch: a wide sketch plus dedup is the baseline that matters. It is over-provisioned for hit-rate. Counters stay well below u16 saturation.\n- TinyLFU admission: keep strict `>` with keep-victim-on-tie. A first-time newcomer that ties a resident accessed once still loses, but this is not where the remaining misses are.\n- SLRU (probation/protected) on top of TinyLFU: a marginal win that is already over-defended. Stale-high keys sit in protected, where the contest never reaches them. Demotion policy, ratio and windows did not change outcomes.\n- W-TinyLFU window: negative on workload-a (an LIRS-style trace) at fixed and adaptive sizes. Few keys come back within a short recency trial.\n- Fast-path coupling: the ≤1 early-stop and early-reject shortcuts are correct only without aging. Any decay or reset proposal must remove them.\n- Workload-a hit-rate has been flat or down for four cycles in a row (0005–0008). Treat it as the ceiling for the TinyLFU+SLRU family.\n"
}
```

### 09:46:19 chat

I wrote a fresh `cross-cycle-insights.md` from the eight-cycle ledger. It uses the four fixed sections in order.

The main conclusions:

- **What paid off:** only one change has given a large hit-rate gain in eight cycles: widening the count-min sketch, which removed collision noise (cycle 0003). Removing the double count on a miss followed by a put added a little more (cycle 0004). Since cycle 0005, winners have mostly just matched the parent's hit-rate.
- **What keeps failing:**
  - **Aging or decay:** tested five different ways, and every one lost or stayed flat.
  - **Contest tweaks:** letting newcomers win ties, ghost-based tie-breaks and margins, and random admission all failed.
  - **Searching deeper for victims:** this evicts young keys that are still useful.
  - **Layout changes:** a small window, an adaptive window, changing the SLRU split, and demoting idle keys all failed.
  - **More sketch-accuracy work:** nothing gained after the wide sketch.
- **Plateau:** hit-rate on workload-a has been flat or down for four cycles in a row (0005–0008). The file treats this as the ceiling for the TinyLFU+SLRU family.
- **Biggest gap:** per-op-time and metadata-size have not been scored in any cycle. Every speed or memory variant has therefore been invisible to winner selection.
- **Other open questions:** how narrow the sketch can get, whether this plateau carries over to other workloads, and two small tests that would only close old threads.
- **Fast-path coupling:** the fast-path shortcuts added in cycle 0005 are only correct without aging. Any future decay proposal has to remove them.


---

- finished: 2026-10-21T09:46:20+00:00
- duration: 22.30s
- status: ok
- input_tokens: 4
- output_tokens: 2123
- cache_read_tokens: 39732
- cache_write_tokens: 12613
- cost_usd: 0.151326
