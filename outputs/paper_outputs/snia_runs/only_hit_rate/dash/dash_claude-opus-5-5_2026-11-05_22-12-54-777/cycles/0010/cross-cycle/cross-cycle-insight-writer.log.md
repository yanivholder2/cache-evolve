# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-11-06T08:21:24+00:00

## timeline

### 08:21:46 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0010)\n\n## Recurring wins\n\n- **Changes that alter structure beat parameter tuning.** The big steps were all structural: the 24/8 topology (c0001), the frequency-aware tail victim (c0002), adding a ghost (c0004), a cache-sized direct-mapped ghost (c0005), ghost hits bypassing the stash (c0007), and the contention gate (c0008). Tuning parameters inside each of these stalled within a cycle or two.\n- **The ghost is the only lever that has moved web_0.** Every web_0 gain in the run came from ghost readmission. hm_0 responds wherever the ghost fires. A cell's response tracks how often the ghost fires there.\n- **Where a credited key lands matters more than how much credit it gets.** Sending ghost hits straight to target turned credit into lasting residency. Bigger credits and longer ghost reach could not do that.\n- **Ageing is safe only when it hits items already judged cold.** Halving a counter on demotion into the stash (c0006) helped. Moving ageing to a boundary where no hit is penalised works. Removing ageing never does.\n- **Protecting warm residents beats protecting newcomers.** The contention gate (c0008) helped small-size hm_0 by refusing bypasses that would displace warm residents. When an un-re-hit ghost arrival is evicted quickly, that acts as a useful filter.\n- **One isolated change per variant works.** Bundled variants (c0002 v001) gave regressions nobody could pin to a cause. Single-change cohorts map each lever clearly.\n\n## Recurring failures\n\n- **Changing insert position after frequency-based eviction exists.** MRU promotion hurt web_0 in c0002 and c0003. Gated demotion was redundant. Front landing is part of how the filter works.\n- **Sweeping parameters of a mechanism that is already used up:**\n  - normal/stash ratio: peaked at about 3/4\n  - tail window width: 8 is optimal, and both narrower and wider lose\n  - ghost size: capacity-sized is enough, and 2× dilutes\n  - ghost bonus: credit 1 already beats the counter-0 tail\n  - fingerprint narrowing: costs web_0\n- **Faster or eviction-count-driven ageing.** It drains Front-landed, counter-1 arrivals inside the scan window, and it hurts small caches most. This has been confirmed in five cycles.\n- **Scan defences aimed at web_0.** A promotion-side TinyLFU gate (c0007) and a sequential-run filter (c0010) both failed. Scan keys never reach the gate, and sequential blocks are actually beneficiaries of the ghost. Removing a key's ghost entry always costs hits.\n- **Gate signals that read only the target window, or only a global threshold.** Three tries failed: a global threshold (c0009), a window histogram (c0010), and per-key refusal strikes (c0010). Each only moved along the trade-off between large-size web_0 and small-size hm_0. Spreading placement to a neighbour bucket did not help either, because that bucket shares the same load.\n- **Process gaps that last.** Per-op-time and metadata-size have gone unmeasured for ten cycles. Overlay checks failed for two of three variants in each of c0008–c0010. No variant reported how often each gate branch fires. Cost arguments, and the mechanisms behind them, are still inferred rather than measured.\n\n## Open questions\n\n- Can a signal about the returner itself (how old its ghost entry is, or whether it was ever hit in target) separate large-size web_0 returners from the counter-1 residents worth protecting on small-size hm_0?\n- Do large-size web_0 returners die in the stash before they ever reach the ghost? If so, would stash retention for ghost-refused keys be the real lever?\n- How often does each gate branch fire, and how do stash-refused returners end up? This instrumentation should come before building any adaptive gate.\n- Large-size hm_0 has been nearly flat since c0007. Is the cause structural, or are keys leaving through paths the ghost never records?\n- Is the ghost/bypass family saturated (two cycles of noise-level winners)? Would a structurally different axis pay off more?\n- What do the incumbent's victim scan, ghost table, and bypass cascade actually cost in per-op-time and metadata-size?\n\n## Per-policy notes\n\n- **Lineage of the incumbent:**\n  - c0001: baseline with the 24/8 topology\n  - c0002: frequency-aware 8-slot tail victim with halve-on-cap\n  - c0005: direct-mapped, capacity-sized ghost\n  - c0006: halving on demotion into the stash\n  - c0007: ghost hits bypass to target Front with counter 1\n  - c0008: bypass gated on a counter-0 or free tail slot (still the effective parent; the c0009 and c0010 winners were noise-equivalent)\n- **Stash:** With admit-everything, the stash is the only way into the protected tier as well as the scan filter, so shrinking it starves reuse on hm_0. Halving on stash hits is its only ageing and must stay. Where cascade victims go (stash or ghost) does not affect hit-rate.\n- **Target / normal tier:** The 8-slot tail scan with slow, halve-on-cap ageing is tuned at its optimum. Front landing is coupled to both the scan filter and counter protection. Any change to landing or ageing must account for this.\n- **Ghost:** Its reach and bonus strength are both used up. The remaining unsettled trade-off is the admission gate between ghost and target, and it depends on cache size. At the small size, protect counter-1 residents. At the large size, admit returners that compete with counter-1 minima.\n- **Traces and sizes:** hm_0 is the most responsive trace in both directions, and small-size hm_0 is where frequency-based mechanisms show the most effect. web_0 barely responds unless the ghost fires, and that happens mainly at the large size.\n"
}
```

### 08:21:52 thinking



### 08:21:52 chat

I wrote `cross-cycle-insights.md` from the cycle 0001–0010 ledger digest, using the four required sections in order and no numbers beyond what the ledger states.

- **What's working:** the big gains all came from new mechanisms, not tuning. These were the 24/8 topology, the frequency-aware tail victim, a ghost sized to the cache, halving counters on demotion into the stash, ghost hits going straight to target, and the contention gate. The ghost is the only thing that has ever moved web_0.
- **What's spent:** tuning the split, the scan window, the ghost's size and bonus, and fingerprint width has stopped paying off. So have insert-position changes, faster ageing, scan defences aimed at web_0, and gates that only look at the target window.
- **Where the run stands:** the ghost/bypass line has given noise-level winners for two cycles (0009 and 0010). The cycle 0008 contention-gated bypass is still the effective parent.
- **Gaps that keep recurring:** per-op-time and metadata-size have gone unmeasured for ten cycles. Overlay checks failed for two of three variants in each of cycles 0008–0010. No variant has reported how often each gate branch fires.
- **Open threads:** a gate based on the returning key itself (such as how old its ghost entry is), whether large-size web_0 returners die in the stash before reaching the ghost, and whether it's time to switch to a different axis altogether.


---

- finished: 2026-11-06T08:21:52+00:00
- duration: 28.44s
- status: ok
- input_tokens: 4
- output_tokens: 2642
- cache_read_tokens: 44036
- cache_write_tokens: 17487
- cost_usd: 0.201559
