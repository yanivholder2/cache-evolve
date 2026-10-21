# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:51:47+00:00

## timeline

### 09:52:05 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0009)\n\n## Recurring wins\n\n- Estimate quality is the only lever that has clearly moved hit-rate. Widening the count-min sketch (0003) gave the one large gain of the run. Dedup of the get-miss-then-put double count (0004) and frequency-aware victim selection (0002) each added a small amount.\n- Lifetime, unaged frequency is the right signal for workload-a. Every comparison with recency or decay has favored it, on every design tested.\n- Strict `>` admission with keep-victim-on-tie has held up. The alternatives tried were tie-admit, ghost tie-break, ghost margin and random admit. All were neutral or lost.\n- Cost-oriented variants keep matching the parent's hit-rate exactly: the fast hash (0001), the narrow u16 CU sketch in non-SLRU (0005), the hash-cached fast path (0005) and the counter cap at 31 (0009). This is good evidence that metadata and latency can be reduced at no hit-rate cost, once those axes are scored.\n- Since 0006, most \"Winners\" have been ties or the smallest loss, not gains. Read the winner label as \"did not regress\".\n\n## Recurring failures\n\n- Every form of aging or recency fails on workload-a: halving short or long, with or without CU, before and after dedup/SLRU (0001, 0002, 0004, 0008), idle discounts and idle demotion (0008), and ghost evidence (0007, 0008). The keys that lifetime counts protect have long reuse distances and are worth more than the stale-high keys that recency would evict.\n- Structural rearrangement does not pay off: the 1% window on both noisy and clean estimates, the adaptive window hill-climber, SLRU ratio, demotion choice and promotion gating. The min-estimate victim sample already acts as a soft protected segment, so the design is over-defended.\n- Victim-sampling depth is used up. Widening the sample evicts young keys, because with unaged counts the lowest estimate marks the youngest resident. Narrowing it raises the victim estimate. Depths from 3 to 8 are equivalent.\n- Further sketch-accuracy work (CU on the wide sketch, more width) adds nothing once collision overcount is gone. Width and CU substitute for each other.\n- Bundled changes have confused attribution more than once (0001 v002, 0008 v003). Change one lever per variant.\n- Per-op-time and metadata-size have gone unscored in all nine cycles. Memory and latency variants can only tie or lose under hit-rate-only fitness.\n\n## Open questions\n\n- Are per-op-time and metadata-size actually in the fitness vector yet? This blocks every remaining lead.\n- If they are scored: how small can the sketch get? Points to test are 5-bit counters (cap 31, expected neutral), 4-bit (cap 15, untested), and narrow-width CU under SLRU, where 8× looked like the floor. A recency tie-break between equal estimates is the only proposed way below that floor.\n- How much does SLRU add in per-hit list work and per-promotion estimate cost compared with the plain-LRU-demotion parent? And does combining SLRU, the narrow CU sketch and the hash-cached fast path stay neutral on hit-rate?\n- Is workload-a's plateau the ceiling for TinyLFU-family policies? Do these conclusions, especially the verdict against aging, carry over to other workloads?\n- Two residual checks remain, at low priority: raw-estimate protected-head demotion without a discount, and halving at a period far beyond 10×capacity.\n\n## Per-policy notes\n\n- **Sketch (count-min):** a wide sketch plus dedup is the baseline. Counts above 31 carry no decision-relevant ordering. Without SLRU, half width with u16 counters and CU is neutral. Under SLRU, a narrow sketch flips some decisions.\n- **Admission contest:** keep strict `>` and keep the victim on ties. Exact ties at warm estimates are rare, and a margin applied only to ghosts is tiny but still negative.\n- **Victim selection:** the 8-way min-estimate sample from LRU/probation is the main source of protection. Its depth is not a lever.\n- **SLRU (probation/protected):** the 0005 gain was marginal, and every internal parameter since then has been inert. Which keys sit in protected is not the binding constraint.\n- **W-TinyLFU window:** tested fixed, adaptive, and on clean estimates. It never helped, because the slots it takes from the main region cost more hits than the window recovers on this LIRS-style trace.\n- **Aging/decay:** negative on every design tested. The fast-path shortcuts from 0005 (early stop and early rejection at estimate ≤ 1) are valid only without aging, so they must be removed if decay is ever reintroduced.\n"
}
```

### 09:52:11 chat

I wrote `cross-cycle-insights.md` in `cycles/0009/cross-cycle/` from the ledger alone, with the four required sections in order. The main conclusion across all nine cycles: hit-rate on workload-a has been flat since cycle 0006, and the only hit-rate lever left untested depends on the cost axes being scored.

- **Recurring wins:** Only estimate quality ever moved hit-rate. The wider sketch in cycle 0003 gave the one large gain; dedup and frequency-aware victim selection each added a little. Long-run (unaged) frequency counts are the right signal for this trace. Every change aimed at speed or memory left hit-rate exactly unchanged.
- **Recurring failures:**
  - Every form of aging or recency evidence lowered hit-rate, on every design tested.
  - Admission windows, the adaptive window, all the SLRU segment parameters and victim-sample depth changed nothing.
  - More sketch-accuracy work added nothing once collision noise was gone.
  - Per-op-time and metadata-size have never been scored, so memory and speed variants can only tie or lose.
- **Open questions:** The blocking one is whether per-op-time and metadata-size are scored yet. If they are, the next tests are how far the sketch can shrink, what SLRU costs per operation, and whether these conclusions hold on workloads other than workload-a.
- **Per-policy notes:** Short summaries for the sketch, admission contest, victim selection, SLRU, admission window and aging. The aging note warns that the cycle-0005 speed shortcuts are only correct without aging and must be removed if decay comes back.

Recent "Winners" should be read as "did not regress", not as gains.


---

- finished: 2026-10-21T09:52:11+00:00
- duration: 24.40s
- status: ok
- input_tokens: 4
- output_tokens: 2334
- cache_read_tokens: 40578
- cache_write_tokens: 13675
- cost_usd: 0.164212
