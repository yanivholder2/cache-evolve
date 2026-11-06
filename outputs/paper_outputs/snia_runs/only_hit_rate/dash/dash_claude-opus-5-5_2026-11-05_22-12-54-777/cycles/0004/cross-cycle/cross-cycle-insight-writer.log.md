# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-11-05T22:40:51+00:00

## timeline

### 22:41:10 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Single, isolated changes on top of a clear parent give readable results. Bundled changes (cycle 0002 v001) left their regressions unattributable.\n- The 24/8 normal/stash split (cycle 0001) is a stable base. Every later winner was built on it, and it helped most at the large size.\n- The frequency-aware 8-slot LRU-tail victim scan with halve-on-cap ageing, counting stash hits (cycle 0002), is the strongest lever so far. It improved all four cells, and it is the only kind of change that has moved the small size.\n- Changes that act inside a bucket (victim choice) or at admission move the small cache and hm_0. Topology changes do not.\n- Admission-side memory (the cycle 0004 ghost ring with re-admit credit) is the first lever to reach hm_0's long-gap reuse. Recency-position changes never did.\n- hm_0 is the most responsive trace in both directions. It is the best place to detect a signal early.\n\n## Recurring failures\n\n- MRU-end promotion has hurt web_0 in cycles 0001, 0002 and 0003, the last time on its own, so MRU landing itself is the cause. Once the tail scan exists, MRU landing also moves new arrivals out of the scan window. Front landing is part of how the filter works.\n- Insert-position changes in general (MRU promotion, gated demotion) are redundant or harmful once eviction reads the frequency counter.\n- Normal/stash ratio sweeps are exhausted. The split peaks at about 3/4. Shrinking the stash starves promotion on hm_0, because with admit-to-stash the stash is the only way into the protected tier.\n- Fixed tail-window sweeps are exhausted. Window 4 lost the filter, and window 16 became coarse LFU and lost recency protection.\n- Faster ageing (a lower cap, or a decrement on every eviction) drains counters before they can protect anything. It is worst at small-size hm_0, where items promoted to Front get aged back to 0 inside the window.\n- Per-op-time and metadata-size have gone unmeasured for four straight cycles, even though every hypothesis asked for them. Cost-reduction variants cannot be judged, and the costs of the scan and ghost ring on every put are unknown.\n\n## Open questions\n\n- Does a larger ghost ring, or a stronger ghost-hit bonus (tested as separate variants), extend the hm_0 gain, and what does it cost on the admission path?\n- Can window width or ageing rate scaled per bucket type (stash vs normal) or by cache size capture the small-hm_0 frequency preference without losing recency elsewhere?\n- Would slower-than-halving ageing, or exempting newly promoted items from ageing, help, given the coupling between Front landing and counters?\n- Is there any lever that moves web_0? Nothing tried so far has a scan-heavy win.\n- Which part of cycle 0003's cheaper config (window vs cap) caused the small-hm_0 loss? This is now low priority.\n- What are the actual per-op-time and metadata costs of the 8-slot scan, the halving, and the ghost ring?\n\n## Per-policy notes\n\n- Baseline (target/probing protected tier): the protected tier gives almost no protection. Promoted items were either evicted quickly (Front landing) or crowded out (small normal tier).\n- 24/8 topology: this is the parent of all later work. It helps at the large size and does little at the small size on its own.\n- Frequency-aware tail victim (window 8, halve-on-cap, stash hits counted): it gains the most at small-size hm_0. Its window, ageing rate and insert positions all sit at local optima, so further tuning of these parameters is spent.\n- Ghost-credited admission (64-entry ghost ring per segment, counter 1 on re-admit): this is the current parent. It gives a small gain on hm_0 at both sizes and leaves web_0 flat. It only reaches reuse gaps just past stash residency, and its one-step bonus is fragile under ageing.\n"
}
```

### 22:41:15 thinking



### 22:41:15 chat

I wrote `cross-cycle-insights.md` in the cycle 0004 cross-cycle output dir, using the four fixed sections in order and built only from the ledger digest.

What the four cycles add up to:

- **What worked:**
  - The 24/8 normal/stash topology is the base every later winner was built on.
  - The frequency-aware 8-slot tail victim choice is the strongest lever so far. It is the only kind of change that has moved the small cache size.
  - The cycle 0004 ghost-credited admission is the first change to reach hm_0's long-gap reuse. It is now the parent.
- **What kept failing:**
  - MRU promotion hurt web_0 in three cycles, even when applied on its own.
  - Insert-position changes have stopped working as a lever.
  - The normal/stash ratio, the tail window width and faster ageing have all been tuned out; none has room left.
  - Per-op-time and metadata-size have not been measured in any of the four cycles, even though every hypothesis asked for them.
- **Still open:**
  - Growing the ghost ring, and giving a stronger bonus on a ghost hit, tested as separate variants.
  - Setting window or ageing separately per bucket type or cache size.
  - Ageing slower than halving.
  - Finding any lever that moves web_0.
  - Measuring what the scan and ring actually cost.


---

- finished: 2026-11-05T22:41:16+00:00
- duration: 24.80s
- status: ok
- input_tokens: 4
- output_tokens: 1943
- cache_read_tokens: 38080
- cache_write_tokens: 9594
- cost_usd: 0.123244
