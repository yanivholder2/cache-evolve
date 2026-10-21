# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-21T14:43:53+00:00

## timeline

### 14:44:12 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Lowering the noise in the estimates gave the only large hit-rate gain in ten cycles. That was the wider count-min sketch in cycle 0003. Every other Winner was marginal or won only by tying the parent.\n- Honest counts helped a little. Dedup of the get-miss-then-put double increment (0004) brought the sketch in line with true access counts, and it should also reduce per-op work.\n- Unaged lifetime frequency is the right signal for workload-a. Every attempt to dilute it lost, and every attempt to sharpen it lost too. The plain access count, weighted equally for every access, already sits at the right balance.\n- Strict `>` with keep-victim-on-tie stays. Every change to tie handling or admission was inert or worse.\n- The sketch is over-provisioned. Width and conservative update substitute for each other. Half width with u16 counters plus conservative update was neutral (0005), except inside SLRU, where about 8× width looks like the floor (0006). Capping counters at 31 is inert on hit-rate (0009), so 5-bit counters should be free.\n- Speed-oriented, semantics-preserving changes reliably tie on hit-rate. Examples are the single fast hash (0001) and the hash-cached fast path with ≤ 1 shortcuts (0005).\n\n## Recurring failures\n\n- **Aging and decay** lost in every form tested: halving at short or long periods, with or without conservative update, on the noisy sketch, on the clean sketch, and on the current SLRU+dedup design (0001, 0002, 0004, 0008). Per-entry idle discounts and idle-based demotion also lost (0008). Variant takeaways keep suggesting aging anyway. Treat that suggestion as refuted.\n- **Recency evidence in the admission contest** failed in all three forms tried: ghost tie-breaks, ghost-hit margins and random admission of warm losers (0007–0008). Exact ties almost never happen at warm estimates, and wider margins churn cold probation keys.\n- **Victim-selection mechanics** are inert or negative. This covers sampling depth from 3 to 8 and wider (0003, 0009), CLOCK-style rotation, and adding the protected head as a candidate (0010). The minimum estimate in the sample does not move, because old probation keys share near-floor counts.\n- **Admission windows** did not pay off. A 1% window was inert on the noisy sketch (0001) and negative on clean SLRU (0006). An adaptive hill-climbing window grew toward LRU and lost (0002).\n- **SLRU internals** are over-defended. Changing the demotion choice, the split ratio, promotion gating or idle demotion never moved hit-rate (0006–0010).\n- **Tilting the contest toward incumbents** also lost: double-weighting hits starves keys that were rejected (0010).\n- **Variants aimed at metadata-size or per-op-time** can only tie or lose, because those dimensions have never been scored in ten cycles.\n\n## Open questions\n\n- Are metadata-size and per-op-time scored yet? This is the binding question. If they are, re-test these four cost leads:\n  - 5-bit counters (cap 31)\n  - a narrow conservative-update sketch on SLRU, with a recency tie-break between equal estimates\n  - the hash-cached fast path\n  - pricing the extra per-op work in SLRU, ghost ring, promotion estimates and rotation\n- Would a 4-bit cap (15) change comparisons? It has never been tested.\n- Do other workloads leave room that workload-a doesn't? On workload-a, TinyLFU+SLRU looks to be at the trace's ceiling.\n- Two minor tests are still open: raw-estimate protected-head demotion alone, and halving with a period far beyond 10×capacity. Both are expected to converge to the parent, so they are low priority.\n\n## Per-policy notes\n\n- **TinyLFU sketch:** the gains come from estimate quality (width, then dedup), and that lever is now exhausted. Width, depth and conservative update overlap, and the counters can be narrow. If any decay or reset is ever reintroduced, cycle 0005's ≤ 1 fast-path shortcuts must be removed, because they assume counts never age.\n- **Admission contest:** keep strict `>` with probation-only victim sampling of about 8, settled by the min-estimate rule. Tested and rejected: tie-admission, jitter, ghost bonuses and changes to sample depth or placement.\n- **W-TinyLFU window:** negative on workload-a, whether fixed or adaptive. Few keys there come back within a short recency trial.\n- **SLRU (probation/protected):** gave a marginal gain over sampled victims alone (0005). It largely duplicates the soft protection that min-estimate sampling already provides. Protected keys are stale in recency but not in count, so no rule based only on frequency can reach them.\n- **Current baseline lineage:** wide sketch with dedup, then SLRU, then a series of hit-rate-neutral Winners (0006–0010). Hit-rate has been flat for six cycles in a row.\n"
}
```

### 14:44:18 chat

I've written `cross-cycle-insights.md` with the four required sections, built only from the ledger. Across ten cycles, workload-a's hit-rate has been flat for the last six cycles in a row, which looks like the ceiling for the TinyLFU+SLRU design.

- **Recurring wins:** The only large hit-rate gain came from widening the count-min sketch (cycle 0003). Removing the double count when a get-miss is followed by a put gave a smaller gain (0004). Long-run access counts that never decay are the right signal for this trace. The sketch can be made smaller (narrower, with counters capped at 31) without hurting hit-rate.
- **Recurring failures:** Every form of count decay lost, including on the current design. Every attempt to let recent use count against long-run frequency also lost: ghost tie-breaks, ghost margins, random admission and idle-based demotion. Changes to how eviction victims are picked, admission windows and the inner rules of the probation/protected split did nothing. Changes aimed at memory size or speed can only tie or lose, because those two measures have never been scored.
- **Open questions:** The main one is whether memory size and per-operation time are scored yet. If they are, the memory and speed ideas from earlier cycles are worth re-testing. Smaller questions are whether a 4-bit counter cap changes decisions, and whether other workloads still have room to improve.
- **Per-policy notes:** I grouped the findings under the sketch, the admission contest, the admission window, the probation/protected split, and the current line of winning variants.

The file says outright that suggestions in variant summaries to add decay again have already been disproved. Later variant summaries kept repeating that suggestion, so I made sure it isn't carried forward as an open lead.


---

- finished: 2026-10-21T14:44:19+00:00
- duration: 25.63s
- status: ok
- input_tokens: 4
- output_tokens: 2431
- cache_read_tokens: 41512
- cache_write_tokens: 14702
- cost_usd: 0.174554
