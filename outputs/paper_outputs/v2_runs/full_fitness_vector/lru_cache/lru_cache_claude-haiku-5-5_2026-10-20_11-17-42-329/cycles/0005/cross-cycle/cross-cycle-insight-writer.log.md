# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-20T11:41:38+00:00

## timeline

### 11:41:45 thinking



### 11:41:50 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Policy-neutral representation changes are the most reliable lever: narrowing arena links, packing flags into spare bits, and sentinel encoding shrink nodes without costing hit-rate (cycles 2, 3, 5).\n- Deferring or bounding recency work mostly helps the put path (miss and eviction tails), not the get path (cycles 2, 4, 5).\n- Keeping hashing and sketch work off the read path lowers read cost; the expense has been the hashing itself, not the admission decision (cycles 3, 4, 5).\n- Admission gating and reuse-aware eviction have produced hit-rate gains on the scan-heavy workload, and those gains come from what is admitted or retained, not from list order (cycles 1, 3).\n\n## Recurring failures\n\n- Per-access overhead on shared paths keeps costing more than it returns: hashing on every access, and extra pointer work in the protected-side rescue that showed up in get as well as put (cycles 3, 4).\n- Changes to arena access or index-conversion structure have not sped up get or put, and hoisted accessors made both slower (cycles 2, 3, 4).\n- Per-node metadata additions without a clear hit-rate gain have not paid off (cycles 1, 5).\n- Protection that rests only on a retained flag, without actually moving the head, recovers little of what list rotation provided (cycles 4, 5).\n- Removing relinks on the hit path did not measurably improve get, so it is not a standalone goal (cycle 2).\n- Per-operation and tail-latency gains from single runs are repeatedly unconfirmed, and several attributions are inferred rather than checked against the diff (cycles 1-5).\n\n## Open questions\n\n- Do the hit-rate gains from admission gating and bounded eviction hold on read-heavy workloads, other traces, and other capacities, or are they specific to the scan pattern?\n- Can the protected-side protection be restored without list surgery on the shared path, using a different recency signal?\n- Are the get and put movements in recent variants real, or layout and run-to-run noise that a re-measure would clear?\n- Can per-access arena lookups be made cheaper without giving back the narrowed link width?\n\n## Per-policy notes\n\n- Segmented / protected-probationary LRU: the flag-only demotion retry cut per-operation cost and gave back a small share of hit-rate; the lost MRU rotation appears to be the protection itself. Credit-based rescue recovered only part of that protection.\n- Second-chance / referenced-bit eviction: a bounded scan traded a small hit-rate loss for a more predictable miss-path cost. The deferred referenced-bit scheme lost some hit-rate on re-referenced keys; a single rescue did not close the gap.\n- Admission filter (frequency sketch): the clearest hit-rate gain in the cohort, by rejecting one-hit keys at capacity. Its cost was hashing on every access. A shared cheap hash and write-only sketch updates lowered read cost, with a small hit-rate loss from dropping read-side popularity.\n"
}
```

### 11:41:55 chat

I wrote `cross-cycle-insights.md` to the cross-cycle output directory for cycle 0005. It has the four required sections in order, and it was regenerated from the inlined ledger digest only. I did not read the previous version.

The main signals across cycles 0001–0005:

- **Recurring wins:** narrowing links and packing flags shrink nodes without costing hit-rate. Deferring or bounding recency work mostly helps the put path. Keeping hashing off the read path lowers read cost. Admission gating and reuse-aware eviction produced the hit-rate gains.
- **Recurring failures:** per-access overhead on shared paths (hashing on every access, extra pointer work in the protected-side rescue). Arena access and index-conversion rewrites have not sped up get or put. Per-node metadata has not paid off without a clear hit-rate gain. A retained flag without moving the head recovers little protection.
- **Open questions:** whether the admission and bounded-eviction gains hold beyond the scan-heavy trace and one capacity, whether protection can return without list surgery, whether the get and put movements are real or noise, and whether arena lookups can get cheaper without losing the narrow link width.
- **Per-policy notes:** grouped by segmented LRU, second-chance/referenced-bit, and the admission filter.

I did not run any other tools or make any commits.


---

- finished: 2026-10-20T11:41:55+00:00
- duration: 17.70s
- status: ok
- input_tokens: 4
- output_tokens: 2598
- cache_read_tokens: 37696
- cache_write_tokens: 10390
- cost_usd: 0.003754
