# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-28T18:44:01+00:00

## timeline

### 18:44:15 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Stash placement has been the strongest lever. In cycle 1, demoted items going to the stash MRU end won and improved every cell. In cycle 3, promotees landing at the target MRU end won. When a moved item lands at its new tier's eviction end, it is effectively thrown away.\n- Protecting reused items from one-hit arrivals keeps helping in tight caches. The two-hit stash exit gate added gains at the small size in both cycles 1 and 2, and survived into cycle 3's winner.\n- Each cycle's winner stacked on the one before (MRU demotion, then the gate, then MRU promotion). Making one targeted change per cycle on top of the previous winner has given steady but small gains.\n\n## Recurring failures\n\n- Every extra admission filter (two-hit gate, frequency comparison, LRU-end fresh insertion) has cost hit-rate on `web_0` at the large size. When targets have room, first-hit promotion is already near-optimal. The only variant that recovered this cell was cycle 3's occupancy shortcut.\n- Easier routes into the target hurt small caches, as with one-hit return of demoted items and the free-slot shortcut. A one-hit item that takes scarce space starts an eviction cascade.\n- Frequency-based admission has been ineffective. Counters stay near zero because moves halve or reset them and promotees enter at the LRU slot. This made the TinyLFU-style comparison behave like plain first-hit promotion.\n- Hard LRU-end insertion of fresh puts starves new keys of their probation hit. It has been dropped.\n- Counter-reset tweaks on demoted items turned out to be near-neutral. Cycle 3 disproved the cycle 2 explanation that the reset caused the `web_0` regression.\n- Per-op-time and metadata-size have never been reported in any cycle. Every cost claim, including the lookup reorder control, is still unverified.\n\n## Open questions\n\n- Does an occupancy-aware gate (skip it when the target has room) stack with MRU-end promotion? Ideally it recovers `web_0` at the large size while keeping the small-size gains.\n- What room threshold (more than one free slot, or an occupancy fraction) keeps the shortcut from hurting tight caches?\n- Should the gate scale with capacity or occupancy instead of staying fixed at two hits?\n- Would delaying a fresh promotee's move to probing (one more target hit) fix the flat `web_0` large-size cell?\n- Could frequency admission work if it compared against the bucket's lowest counter and counters survived moves?\n- Does target-first lookup order lower get latency? Its hit-rate is identical to baseline, so this needs per-op-time measured.\n\n## Per-policy notes\n\n- The ledger covers one lineage: a bucket-local, two-tier design with a target tier and a small LRU stash. All levers act on the stash and its boundary with the target.\n- Cache size splits the results: changes that help in tight caches (gates, stricter admission) tend to hurt in roomy ones, and the reverse. Results split across cells more often than they move together.\n- `hm_0` responds to stash-filtering levers at the large size. `web_0` at the large size is the persistent hard cell and favors permissive admission.\n"
}
```

### 18:44:22 chat

I wrote `cross-cycle-insights.md` from the three-cycle ledger, using the four required sections in order.

- **Recurring wins:** Where an item lands when it moves in or out of the stash has been the strongest lever. Cycle 1's winner sent demoted items to the stash MRU end, and cycle 3's winner sent promoted items to the target's MRU end. The two-hit gate keeps helping at the small cache size. Each cycle's winner built on the previous one with a single change, and that has given steady but small gains.
- **Recurring failures:**
  - Every extra admission filter has cost hit-rate on `web_0` at the large size. The only change that recovered it was cycle 3's v002, which skips the gate when the target has a free slot.
  - Easier routes into the target hurt small caches.
  - Frequency-based admission has had no effect, because the counters stay near zero.
  - Resetting or keeping counters on demoted items made almost no difference.
  - Per-op-time and metadata-size have never been measured in any cycle, so every cost claim is still unverified.
- **Open questions:**
  - Does that free-slot gate stack with the cycle 3 winner?
  - What threshold for "the target has room" protects small caches?
  - Should the gate scale with how full the cache is?
  - Would requiring one more target hit before a promoted item is probed fix `web_0` at the large size?
  - Can a fixed frequency comparison work?
  - Does the lookup reorder actually lower latency?
- **Per-policy notes:** The ledger covers only one design (a target tier plus a small stash), so I grouped these notes by cache size and trace instead. Changes that help small caches tend to hurt large ones, and the reverse. `hm_0` responds to stash filtering at the large size, while `web_0` at the large size stays the hard cell and does best with permissive admission.

The file contains no numbers, only qualitative statements.


---

- finished: 2026-10-28T18:44:22+00:00
- duration: 21.27s
- status: ok
- input_tokens: 4
- output_tokens: 1853
- cache_read_tokens: 36797
- cache_write_tokens: 8082
- cost_usd: 0.109091
