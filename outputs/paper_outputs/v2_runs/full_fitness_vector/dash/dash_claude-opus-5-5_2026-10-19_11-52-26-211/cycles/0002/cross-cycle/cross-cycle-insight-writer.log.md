# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-10-19T12:10:18+00:00

## timeline

### 12:10:35 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Treating the stash as a probationary queue ordered by proven reuse is the main hit-rate lever. It worked twice: demotees went to the stash MRU end (cycle 1) and fresh puts went to the stash LRU end (cycle 2). The two gains added up.\n- Checking the target, then probing, then the stash is a free get-path gain with a clear mechanism, because most hits come from normal buckets. It is settled, so don't revisit it.\n- Single-site changes along the promotion/insert path compose cleanly. Stash insert position, lookup order and promotion destination are independent, and combining them reproduced the expected cache state.\n- No variant so far has moved metadata-size. Placement and order changes come free on that dimension.\n\n## Recurring failures\n\n- Tweaks to placement when a stash item is promoted to the target. Balanced placement (cycle 1) rarely triggered. MRU placement (cycle 2) lowered hit-rate slightly and lengthened the get tail. In tiny target buckets, hot placement on the way up evicts residents that already proved themselves, and it skips the climb inside the bucket. Placing promotions at the Front works as a useful admission filter.\n- \"Winner\" labels have included neutral and negative variants. Judge results by real metric movement, not by the label.\n- Latency deltas compared against the baseline have repeatedly shown up on code paths the diff never touched, so they look like environment or run noise. Only credit per-op-time changes that have a code-level mechanism, and measure the baseline in the same run.\n\n## Open questions\n\n- Has fresh-put-at-stash-LRU been confirmed on top of the combined parent (lookup order + Back-demotion)? It is expected to carry over, but that hasn't been verified.\n- Higher hit-rate pushes more traffic onto the slowest hit path: stash lookup → promote_to_target → possible cascade. Can that path be made cheaper to win back get p50?\n- Would stricter admission into probing (for example, requiring a second target hit) protect the hottest tier?\n- How do hits actually split between the stash and the target/probing buckets? This is still unmeasured.\n- Would balanced placement at initial put, or on target-hit promotion into a neighbour's spare space, trigger often enough to matter?\n- Front insertion seems to add a small put-p50 cost from shifting the bucket. Is that real or noise?\n\n## Per-policy notes\n\n- Stash (most of the capacity): its eviction order dominates hit-rate. It rewards scan resistance: demotees go to the MRU end, new arrivals to the LRU end. It is also the costliest place to look up, so check it last.\n- Target buckets: they are tiny, so placement there is very sensitive. Keep incoming promotions at the Front/LRU end so residents with hits stay safe.\n- Probing buckets: they serve a large share of hits. Climbing within the bucket keeps them selective, and stricter admission is the next lever to try.\n- Get path vs. put path: hit-rate gains tend to cost some get-path latency through the extra stash→target promotions. Front inserts may add a small put-path cost.\n"
}
```

### 12:10:40 chat

I wrote `cross-cycle-insights.md` in the output dir with the four required sections, built only from the two cycles in the ledger.

- **Recurring wins:** The stash's eviction order is the main hit-rate lever. Treating it as a probationary queue paid off twice: demoted items go to the recently-used (MRU) end, and fresh puts go to the least-recently-used (LRU) end. The two gains added up. Checking target, then probing, then stash is a free lookup gain and is settled. The changes combine cleanly, and none has moved metadata-size.
- **Recurring failures:** Changing where items land when promoted from the stash to the target didn't work in either cycle. In cycle 1 it rarely triggered; in cycle 2 it lowered hit-rate. Some variants labelled "Winner" were neutral or negative, so judge by the actual metric change. Latency changes against the baseline keep appearing on code the diff didn't touch, so they shouldn't be credited without a clear cause.
- **Open questions:**
  - Does putting fresh puts at the LRU end still help once it's combined with the new parent?
  - Can the stash→target promotion path be made cheaper?
  - Would stricter admission into probing (such as a second hit) help?
  - How do hits split between the stash and the target/probing buckets?
  - Would balanced placement at put time or on target-hit promotion trigger often enough to matter?
  - Is the small put-time cost of inserting at the front real or noise?
- **Per-policy notes:** These are grouped by stash, target buckets, probing buckets, and the get path versus the put path.


---

- finished: 2026-10-19T12:10:41+00:00
- duration: 22.94s
- status: ok
- input_tokens: 4
- output_tokens: 1694
- cache_read_tokens: 34766
- cache_write_tokens: 7259
- cost_usd: 0.098921
