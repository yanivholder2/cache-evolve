# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-11-05T22:48:50+00:00

## timeline

### 22:49:06 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Making one isolated change per variant on the current parent has given a clear Winner or a clean negative result in every cycle (cycles 1, 3, 4, 5). Variants that bundled several changes (cycle 2 v001) could not be traced back to a single cause.\n- The lineage so far is: 3/4 normal-stash topology (c1), then a frequency-aware 8-slot LRU-tail victim with halve-on-cap ageing (c2), then ghost-credited admission (c4), then a direct-mapped, capacity-sized ghost (c5). Each step stacked on the previous one with no regression.\n- Changes that act inside a bucket (victim choice, admission credit) move the small cache size. Topology alone moved only the large size.\n- Admission-side memory (the ghost) is the only lever that has reached hm_0's long-gap reuse. Ghost reach (capacity) mattered more than bonus strength.\n- Making the ghost lookup O(1) improved hit-rate and also argues for lower cost. Structural simplifications that also add reach are worth preferring.\n\n## Recurring failures\n\n- MRU or Back landing on promotion: it hurt web_0 in c2 (bundled with other changes) and again in c3 (applied alone). Front landing is part of how the tail-scan filter works, because it keeps arrivals inside the window.\n- Insert-position changes once eviction reads the counter: gated demotion (c3) was redundant with the scan.\n- Tuning the tail-scan window: narrower (c3) loses the frequency filter, wider (c4) turns the policy into coarse LFU. Window 8 is the peak.\n- Counter ageing: faster ageing (c3 cap 7, c4 decrement-per-eviction) drains protection, and removing the stash-hit halving (c5) makes items stick in target. Counters must keep ageing at roughly the current rate.\n- Normal/stash ratio sweeps past 3/4: shrinking the stash starves promotion on hm_0. The ratio has peaked.\n- Narrowing the ghost fingerprint: false matches cost web_0.\n- Measuring cost: per-op-time and metadata-size have been requested every cycle and reported in none (5 cycles). Every cost claim is still argued from code.\n\n## Open questions\n\n- Does a +2 ghost-hit bonus add on top of the capacity-sized ghost, now that the ghost fires more often, if fingerprints stay full width?\n- Does a ghost larger than capacity (e.g. 2×), or a different admission rule, reach the large-size hm_0 reuse, which every ghost change so far has left untouched?\n- Can stash ageing be moved rather than removed (age on insertion, or reset/halve a demoted item's counter on entry) to protect ghost credits without stickiness?\n- Would window width or ageing rate scaled per bucket type or by cache size capture the small-hm_0 frequency preference without losses elsewhere?\n- Is there any lever for web_0? Nothing tried in five cycles has produced a gain there.\n- What are the actual per-op-time and metadata-size of the scan, the halving, and the ghost table?\n\n## Per-policy notes\n\n- Topology (normal/stash split): 3/4 normal is the best split. Stash entry is the only path into the protected tier, so stash size bounds promotion. The split is spent as a lever.\n- Tail-scan victim (frequency filter): 8-slot window plus slow halve-on-cap ageing is near optimal. Its gains are concentrated in small-size hm_0, and it depends on Front landing.\n- Promotion/demotion placement: Front landing is required. MRU variants and counter-gated placement have been neutral or harmful since the filter was added.\n- Ghost admission: reach matters more than bonus. Capacity-sized direct-mapped is the current best. Gains appear only at small size on hm_0. Fingerprint width is a real cost on web_0.\n- Traces: hm_0 is the most responsive trace in both directions. web_0 is nearly inert and reacts mainly to harm: MRU landing and fingerprint false matches.\n"
}
```

### 22:49:12 thinking



### 22:49:12 chat

I wrote `cross-cycle-insights.md` in the cycle 0005 cross-cycle folder, built from scratch from the five-cycle ledger with the four required sections in order.

- **Recurring wins:** Four changes have stacked without any regression: the 3/4 normal/stash split, the frequency-aware 8-slot tail victim, ghost-credited admission, and the direct-mapped ghost sized to the cache. Testing one change per variant gave readable results every cycle. The ghost is the only lever that has reached hm_0's long-gap reuse, and it worked through reach, not bonus strength.
- **Recurring failures:** Several approaches keep not paying off:
  - landing promoted items at the MRU end, which hurt web_0 twice;
  - changing insert position once eviction already reads the counter;
  - changing the tail-scan window width or the ageing rate;
  - sweeping the normal/stash ratio past 3/4;
  - narrowing the ghost fingerprint.

  Per-op-time and metadata-size have been asked for in every cycle and reported in none, so all cost claims are still argued from the code rather than measured.
- **Open questions:**
  - whether a +2 ghost bonus adds on top of the cache-sized ghost;
  - whether a ghost larger than the cache reaches large-size hm_0;
  - whether stash ageing can be moved rather than removed;
  - whether window or ageing settings should differ by bucket type or cache size;
  - whether any lever helps web_0;
  - what the scan, halving and ghost table actually cost.
- **Per-policy notes:** Grouped by topology, tail-scan victim, promotion/demotion placement and ghost admission, plus notes on how each trace responds.

All numbers come from the ledger. Everything else is described in qualitative terms.


---

- finished: 2026-11-05T22:49:13+00:00
- duration: 22.72s
- status: ok
- input_tokens: 4
- output_tokens: 2124
- cache_read_tokens: 39136
- cache_write_tokens: 10669
- cost_usd: 0.135675
