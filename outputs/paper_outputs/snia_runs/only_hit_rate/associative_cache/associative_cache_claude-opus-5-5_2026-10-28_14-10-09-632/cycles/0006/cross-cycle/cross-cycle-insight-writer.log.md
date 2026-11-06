# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-28T15:11:42+00:00

## timeline

### 15:12:01 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Cold insertion of new items is the most durable idea we have. The traces reward ranking new items below reused ones. It won in LRU (0001) and worked even better when moved to CLOCK as a probation FIFO in front of main (0004 onward). Each later Winner (0005, 0006) is a refinement of that structure.\n- A hit during probation is the only promotion signal that has held up. Changes that keep this strict filter (only hit-in-probation items get into main) have gained on both traces. Changes that loosen it have caused trades or regressions.\n- Frequency-aware CLOCK (GCLOCK) improved on its parent in every cycle where it was the focus (0002, 0004, 0006), and its advantage grows with bucket size. It is the only lineage that has moved the large-bucket cells (0004 web_0, 0006 hm_0).\n- Resetting the counter to 0 on promotion is a safety property: it has stopped several otherwise-harmful main-side changes from churning main (0005, 0006). Keep it.\n- Gains on the main side come from how well main *ranks* its residents (a higher counter ceiling in 0006), not from making main bigger or easier to enter.\n\n## Recurring failures\n\n- A single fixed rule in the LRU lineage (LIP/BIP mixes, floors, PSEL feedback, graded promotion) has traded hm_0 against web_0 in every cycle from 0002 to 0004. Probation and protected regions compete for the same slots there, so the lineage is stalled.\n- Ghost/warm admission has failed in both lineages, both undersized (0003) and main-sized (0005). It brings back hm_0's near-miss reuse but re-admits web_0's single-return blocks. This is a trace split, not a sizing problem. Retire it.\n- Lowering the bar for entering main, or adding counter mass at entry (ghost match, a non-zero counter on promotion), speeds up churn in main and hurts web_0 (0005). Several variants' takeaways keep re-suggesting this; the results refute it.\n- Per-bucket feedback state that is shorter than the reuse distance it is meant to detect (sample rings, ghost rings) loses the signal before it arrives (0003). Symmetric PSEL updates also vote on the wrong question (0004).\n- The probation-depth lever is exhausted. It helped once (the 0005 floor), and pushing past about half the bucket in small buckets or past a quarter in large buckets took away main capacity and lost hit-rate (0006).\n- Per-op-time and metadata-size have never been measured in six cycles. The fitness vector does not include them, so this is a harness gap. Costs that are growing but unseen: mid-vector inserts, linear scans on the miss path, and GCLOCK sweeps, which got worse with the higher ceiling.\n\n## Open questions\n\n- Can main add a recency signal next to frequency (for example a recent-hit bit that sweeps clear before decrementing) to recover hm_0's near-miss reuse at the large size, without reviving the web_0 pollution seen with ghost admission?\n- What still limits web_0, especially at the small size? Depth, admission strength and counter resolution have each been ruled out as the full explanation.\n- Is there any gain left in small-bucket probation depth between the current floor and the point where it failed, or is that lever finished?\n- Would leader/follower set dueling on the probation share pick the right depth for each workload, where fixed values and per-bucket self-feedback have both failed?\n- What do the GCLOCK sweeps actually cost in worst-case latency once per-op-time is measured?\n\n## Per-policy notes\n\n- **ClassicLRU / LIP-BIP lineage (0001–0004):** cold insertion was a clear win at first. After that, every extension traded one trace for the other. Bounded promotion while inserting at MRU is harmful. The floor and BIP do not stack. Only a corrected asymmetric PSEL was ever proposed as worth keeping, and it is low priority.\n- **CLOCK / GCLOCK (0001–0003):** coarse recency does not cost hit-rate. A multi-bit counter helps, mostly in large buckets. Ghost admission on bare GCLOCK added pollution.\n- **GCLOCK + probation FIFO (0004–0006, current main lineage):** this is the best design so far and the current parent (floor 64, quarter fraction in large buckets, ceiling 7, reset to 0 on promotion). Up to its limits, probation depth is not zero-sum here, because a frequency-ranked main needs fewer slots than an LRU protected segment. The next lever is victim selection inside main, most likely adding recency to the frequency counter.\n"
}
```

### 15:12:09 thinking



### 15:12:09 chat

I've regenerated `cross-cycle-insights.md` from the six-cycle ledger, with the four required sections in order.

**What the file concludes:**
- **Lineage:** The design that has held up is GCLOCK with a probation FIFO in front of main, which has been the main lineage since cycle 0004. It traces back to the cold-insertion idea from cycle 0001. It's the only line that has moved the large-bucket results, and the current parent is cycle 0006 v003.
- **What to keep:** Only items hit while in probation should get into main, and their counter should reset to 0 when they do. Every change that loosened either of these did worse.
- **What to drop:** Four directions should be retired:
  - adding fixed rules to the LRU lineage, which traded one trace (`hm_0`) against the other (`web_0`) every cycle;
  - ghost/warm admission, which failed in both lineages and at both sizes tried;
  - making it easier to get into main or giving promoted items a non-zero counter, which several variants still suggest even though the results refute it;
  - more probation-depth tuning, which has hit its limits.
- **Next lever:** Choosing which item to evict from main, most likely by adding a recency signal alongside the frequency counter. The main unresolved question is what still holds back `web_0`.
- **Measurement gap:** Speed per operation (per-op-time) and metadata size have never been measured in six cycles. This is a harness gap, because they aren't part of the fitness vector. Sweep costs keep growing unseen, and they went up again with this cycle's higher counter ceiling.

As instructed, the file is a fresh projection of the ledger. It doesn't draw on the previous version and contains no invented numbers.


---

- finished: 2026-10-28T15:12:09+00:00
- duration: 27.31s
- status: ok
- input_tokens: 4
- output_tokens: 2276
- cache_read_tokens: 40100
- cache_write_tokens: 11857
- cost_usd: 0.148412
