# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-29T13:32:53+00:00

## timeline

### 13:32:57 thinking



### 13:33:09 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Admission gating based on reuse evidence has improved hit-rate in every cycle. The stash reuse gate from 0001 became the lean threshold-only gate in 0002, which became the full-target occupancy guard in 0003. The gains come from deciding *who enters* a tier, not *where* an item lands or *how quickly it returns*.\n- Fewer multi-bucket moves (promotions, demotion cascades, forced demotions into stash) means better hit-rate and better put/get latency together. Every clear mechanistic Winner reduced stash churn, and every loser added to it.\n- Keeping the common target-hit path free of bookkeeping pays off. Removing per-hit counter writes recovered median get. The cheapest form of each guard (a threshold instead of a counter plus a comparison) kept most of the hit-rate gain.\n- Large cache sizes, and hm_0 especially, are where reuse-aware changes win or lose hit-rate. Small-cache cells barely respond or move the opposite way.\n\n## Recurring failures\n\n- Placement based only on free space or capacity (balanced or free-slot placement, even when limited by a reuse flag) never moved hit-rate under pressure. It is dropped.\n- Extra \"protect reused items\" mechanisms stacked on the gate add little. MRU demotion landing and reuse flags mostly duplicate what the gate already captures.\n- Anything that makes coming back cheaper (re-admission credits, faster re-promotion) increases churn between reused residents and evicts fresh items still waiting for their second hit. It lowered hit-rate in every cell.\n- Latency-only \"wins\" have been unreliable for three cycles straight. Variants whose diffs could not explain their latency changes still moved a lot, and none of them has been checked against a re-run of the baseline.\n- Narrowing `lfu_counter` alone does not shrink Item. `timestamp: Instant` sets the size, and any field of 8 bytes or less disappears into its alignment padding.\n\n## Open questions\n\n- How much noise does per-op-time carry? Re-run an identical baseline before trusting any more latency-only differences. This has been raised every cycle and is still not done.\n- Should gate thresholds scale with cache or stash size? A fixed threshold helps large caches and slightly hurts small ones, because a key can't collect enough hits during a short stash stay.\n- Can a write-free or size-gated victim guard win back the large-cache hits that were lost when the victim comparison was removed?\n- Can `timestamp` become a tick of 4 bytes or less, or be dropped for policies that never read it, so that Item shrinks (to about 24 bytes) without hurting hit-rate?\n- Does better victim choice inside a full target add anything beyond the door guard?\n- What limits web_0 at the large size, now that placement tweaks have been ruled out?\n\n## Per-policy notes\n\n- Dash stash → target admission: this is the productive line. The current baseline is a threshold-only reuse gate plus a stricter threshold for entering a *full* target. Hit-rate gains come from here.\n- Dash target → probing promotion (hottest-in-target rule): insert position and promotion are coupled, because landing position sets how quickly an item climbs on its next hit. Changes here easily add churn. Requiring reuse before promotion is still untested.\n- Dash demotion/landing (MRU, Back positions): gives a small, inconsistent benefit once the gate exists. Low priority.\n- Item layout / metadata-size: unchanged after three cycles. The `u8` counter is in place as a prerequisite, and the next lever is `timestamp`.\n"
}
```

### 13:33:15 chat

I wrote `cross-cycle-insights.md` for cycles 0001–0003, built only from the ledger digest and using the four required sections in order.

- **Recurring wins:** Hit-rate improves when the change decides *who enters* a tier based on reuse evidence. That line runs from the 0001 stash gate, to the lean threshold-only gate in 0002, to the full-target occupancy guard in 0003. Changes that cut multi-bucket moves (stash churn) help hit-rate and latency together. Keeping bookkeeping off the common hit path protects median get. Large caches, hm_0 especially, are where hit-rate is won or lost.
- **Recurring failures:**
  - Placement based only on free space never moved hit-rate.
  - Extra "protect reused items" mechanisms stacked on the gate add little.
  - Making it cheaper for items to come back increases churn and lowered hit-rate in every cell.
  - Latency-only "wins" can't be trusted yet.
  - Narrowing the counter alone can't shrink Item, because `timestamp` sets its size.
- **Open questions:**
  - How noisy per-op-time is. No one has re-run an identical baseline yet, even though every cycle has asked for it.
  - Whether gate thresholds should scale with cache size.
  - Whether a cheap victim guard could win back the lost large-cache hits.
  - Whether `timestamp` can be narrowed or removed.
  - Whether better victim choice inside a full target adds anything.
  - What limits web_0 at the large size.
- **Per-policy notes:** The ledger covers only the Dash design, so I grouped these by mechanism instead of by eviction policy: stash → target admission, target → probing promotion, demotion/landing, and Item layout.


---

- finished: 2026-10-29T13:33:16+00:00
- duration: 23.07s
- status: ok
- input_tokens: 4
- output_tokens: 2011
- cache_read_tokens: 37134
- cache_write_tokens: 8623
- cost_usd: 0.116647
