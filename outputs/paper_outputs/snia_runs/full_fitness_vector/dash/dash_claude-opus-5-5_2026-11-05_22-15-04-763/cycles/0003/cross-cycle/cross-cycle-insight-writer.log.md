# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-11-05T22:39:23+00:00

## timeline

### 22:39:38 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Retention-extending changes reliably raise hit rate: the two-hit gate (c1), MRU demotion into stash (c2, c3, also ungated), and probing overflow (c2). Every evaluated variant on this line improved hit rate on all trace/size cells except the one that removed first-hit reordering.\n- hm_0 at the large size is consistently the most sensitive cell — biggest gains from gate and MRU demotion, biggest loss when retention is weakened. Use it as the primary signal for retention ideas.\n- Cutting demotion cascades is the one lever that has recovered tail latency: probing overflow on top of gate + MRU demotion reversed most of the `put` p99 regression (c3).\n- Reusing existing per-item metadata (e.g. the unused `lfu_counter`) adds state without growing metadata size.\n- Current best base: gate + MRU demotion + probing overflow (c3 v002) — best combined hit rate and tail profile so far, still no Winner.\n\n## Recurring failures\n\n- No cycle has produced a Winner; the persistent blocker is `get` latency (esp. p50) regressing versus baseline once stash retains items longer.\n- Latency fixes on the first-hit path don't pay: skipping the first-hit MRU move (c2) gave negligible p50 help, no p99 help, and lost hit rate.\n- Lookup reordering doesn't pay: checking normal tier before stash (c3) left hit rate flat and worsened `get` p50/p99 — stash hits are too large a share of hits.\n- Blaming the gate for `get` slowdown is wrong: ungated MRU demotion lands in the same slower `get` band (c3). Cost tracks stash-hit traffic and the promote/demote chain.\n- Stacking promotion-placement and demotion-placement changes gives sub-additive hit-rate gains; overflow's value on top of MRU demotion is mostly latency, not retention.\n- Build infra (Maven/Gradle HTTP 429) killed variants in c1; failures of that kind say nothing about the idea.\n\n## Open questions\n\n- Can the stash-hit promotion path (remove → promote → demote cascade) be made cheaper — skipped, deferred, or batched when not needed — without losing retention?\n- Is the remaining gap to dominating baseline only `get` p50? How much is left after cascade reduction?\n- Does probing overflow on the ungated MRU-demotion branch make that cheaper line competitive?\n- Can the probing-overflow lookup penalty (extra bucket probed) be removed, e.g. by migrating items back to target when space frees?\n- Is any residual `put` p99 regression driven by stash fullness or benchmark noise? Still not cleanly separated.\n- H03-style balanced promotion between target/probing buckets was never evaluated in isolation.\n\n## Per-policy notes\n\n- Dash (stash + normal tier): working theory — a small normal tier gets flushed by weakly re-referenced items — has been supported every cycle. Hit-rate levers: admission gating, MRU demotion, overflow placement. Latency drivers: stash-hit promotion chain and demotion cascade count; stash occupancy pressure hurts `put` tails. Keep stash-first lookup order.\n"
}
```

### 22:39:46 thinking



### 22:39:46 chat

I wrote `cross-cycle-insights.md` with the four required sections, built only from the three ledger cycles. None of the three cycles produced a Winner.

- **Recurring wins:** Changes that keep items in stash longer kept raising hit rate: the two-hit gate, demoting into the stash MRU end (with or without the gate), and sending promotions into free probing space. hm_0 at the large size reacts most strongly to these changes. Cutting demotion cascades is the only change so far that has won back tail latency. The best base so far is gate plus MRU demotion plus probing overflow (cycle 3, v002).
- **Recurring failures:** `get` latency keeps getting worse than baseline, and that is what blocks a Winner. Speeding up the first stash hit didn't help, and neither did changing the lookup order. The gate isn't the cause of the slowdown; the cost follows how much traffic hits stash and goes through the promote/demote chain. Gains from combining promotion and demotion placement add up to less than the sum of each alone. The build failures in cycle 1 (HTTP 429 rate limits) say nothing about the ideas they blocked.
- **Open questions:**
  - Can the stash-hit promotion chain be made cheaper?
  - Is the remaining gap to baseline only `get` p50?
  - Would probing overflow make the ungated MRU-demotion branch competitive?
  - Can the extra bucket lookup for items placed in probing be removed?
  - Is the leftover `put` p99 regression from stash fullness or benchmark noise?
  - Balanced promotion between target and probing buckets (H03) has never been tested on its own.
- **Per-policy notes:** These cover Dash only, since it is the only policy in the ledger. They list what drives hit rate, what drives latency, and the advice to keep checking stash first on lookup.


---

- finished: 2026-11-05T22:39:46+00:00
- duration: 23.32s
- status: ok
- input_tokens: 4
- output_tokens: 1838
- cache_read_tokens: 36882
- cache_write_tokens: 8135
- cost_usd: 0.109636
