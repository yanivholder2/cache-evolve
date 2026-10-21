# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:40:24+00:00

## timeline

### 09:40:41 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Estimate quality was the only lever that produced a clear hit-rate gain. Widening the sketch to remove collision noise (cycle 3) beat every contest, structure and recency tweak tried in cycles 1–7.\n- Honest counts helped a little. Deduplicating the get-miss-then-put double increment (cycle 4) was a small but real win. Every later variant builds on wide sketch + dedup.\n- Lifetime frequency is the right signal for workload-a. Unaged counts plus strict `>` with keep-victim-on-tie held up against every challenger.\n- Since cycle 5, wins have mostly come from holding the parent's hit-rate, not improving it. SLRU (cycle 5), frequency-based demotion (cycle 6) and the ghost tie-break (cycle 7) each won marginally or by tying. Hit-rate on workload-a has been on a plateau for three or more cycles.\n- Cost-oriented changes keep hit-rate unchanged: the fast hash (cycle 1), the hash-cached fast path, and the narrow u16 conservative-update sketch in the non-SLRU parent (cycle 5). Their real benefit has never been measured.\n\n## Recurring failures\n\n- Aging/decay lost or was flat in cycles 1, 2 and 4, at short, long and very long periods, and on both noisy and clean sketches. Early losses were confounded by collision noise and conservative update. It has not been re-tested on the SLRU+dedup design.\n- Tie and contest tweaks are inert or harmful. Tie-admission (cycle 3), ghost exact-tie wins (cycle 7) and random admission jitter (cycle 7) either churn cold probation keys or never fire, because exact warm ties are rare with lifetime counts.\n- Admission windows and window adaptation were negative in cycles 1, 2 and 6. Few keys return within a short recency trial, and a hill-climber drifts toward an LRU-like layout that this trace punishes.\n- Victim-sampling width is exhausted. Deeper samples evict young residents, because the lowest unaged estimate marks the youngest key, not the coldest (cycles 2–3).\n- SLRU-internal tweaks (demotion policy, segment ratio) move entries around without changing which keys get evicted, because the min-estimate sample already acts as a soft protected segment.\n- Further sketch-accuracy work (conservative update on a wide sketch) is redundant. Width and conservative update fix the same error, so their gains don't add up.\n- An evaluation gap has persisted in all 7 cycles: only hit-rate is scored. Per-op-time and metadata-size have never been measured, so memory and latency variants can at best tie and are often counted as losses.\n\n## Open questions\n\n- Is aging still negative on the current SLRU+dedup design? This is one decisive test. It must remove the ≤ 1 fast-path shortcuts first.\n- Can a recency signal act where stale-high keys actually live (protected-segment demotion), or through a margin-based ghost bonus instead of an exact tie?\n- Once metadata-size and per-op-time are scored, which combination dominates (narrow CU sketch, hash-cached fast path, SLRU), and what does SLRU's per-hit work cost?\n- How narrow can the sketch get under SLRU? Width 8× looks like the floor. Could a recency tie-break between equal estimates push it lower?\n- Is the current plateau simply workload-a's ceiling for TinyLFU-family policies?\n\n## Per-policy notes\n\n- **TinyLFU contest:** keep strict `>` with keep-victim-on-tie. The bottleneck was estimate noise, not the comparison rule.\n- **Count-min sketch:** wide + dedup is the baseline. It is over-provisioned without SLRU (half width with u16 + conservative update was neutral). Under SLRU, narrowing costs a little, because probation-only sampling compares the newcomer against low-count entries where leftover collision error flips decisions.\n- **W-TinyLFU window:** negative on workload-a in every form tried (fixed 1%, adaptive, and on top of SLRU).\n- **SLRU (probation/protected):** a marginal gain over sampled LRU, then flat under every internal tweak. Protected keys are reachable only through demotion, so neither the contest nor the victim sample ever touches stale-high residents.\n- **Sampled-victim LRU:** a small win at 8-way min-estimate. A wider sample hurts.\n- **Ghost/recency history:** adds unpriced metadata and has had no effect so far through exact-tie use.\n"
}
```

### 09:40:46 chat

I've written `cross-cycle-insights.md` with the four required sections, built only from the ledger for cycles 1–7.

- **Hit-rate is flat.** The only clear gain on workload-a came from widening the sketch in cycle 3, which removed collision noise. Removing the double count on get-miss-then-put in cycle 4 added a small gain. Since cycle 5, winners have mostly just matched their parent's hit-rate.
- **Failures that keep repeating:**
  - **Aging:** lost or was flat in three cycles, though never re-tested on the current probation/protected (SLRU) design.
  - **Tie and admission-rule tweaks:** inert or harmful.
  - **Admission windows:** negative in every form tried.
  - **Wider victim sampling:** hurts, because it evicts young keys.
  - **Changes inside the SLRU layout:** reshuffle entries without changing which keys get evicted.
- **Scoring gap:** per-op-time and metadata-size have not been scored in any of the seven cycles. So variants aimed at speed or memory can at best tie, and are often counted as losses.
- **Open questions:**
  - Is aging still negative on the current design?
  - Can a recency signal be applied to protected-segment demotion, or as a margin in the admission contest rather than an exact tie?
  - Which combination of changes wins once speed and memory are scored?
  - How narrow can the sketch get under SLRU?
  - Is this plateau simply the ceiling for TinyLFU-style policies on this trace?


---

- finished: 2026-10-21T09:40:47+00:00
- duration: 22.41s
- status: ok
- input_tokens: 4
- output_tokens: 2196
- cache_read_tokens: 38847
- cache_write_tokens: 11867
- cost_usd: 0.146641
