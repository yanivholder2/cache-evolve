# Cross-Cycle Insights

## Recurring wins

- Estimate quality was the only lever that produced a clear hit-rate gain. Widening the sketch to remove collision noise (cycle 3) beat every contest, structure and recency tweak tried in cycles 1–7.
- Honest counts helped a little. Deduplicating the get-miss-then-put double increment (cycle 4) was a small but real win. Every later variant builds on wide sketch + dedup.
- Lifetime frequency is the right signal for workload-a. Unaged counts plus strict `>` with keep-victim-on-tie held up against every challenger.
- Since cycle 5, wins have mostly come from holding the parent's hit-rate, not improving it. SLRU (cycle 5), frequency-based demotion (cycle 6) and the ghost tie-break (cycle 7) each won marginally or by tying. Hit-rate on workload-a has been on a plateau for three or more cycles.
- Cost-oriented changes keep hit-rate unchanged: the fast hash (cycle 1), the hash-cached fast path, and the narrow u16 conservative-update sketch in the non-SLRU parent (cycle 5). Their real benefit has never been measured.

## Recurring failures

- Aging/decay lost or was flat in cycles 1, 2 and 4, at short, long and very long periods, and on both noisy and clean sketches. Early losses were confounded by collision noise and conservative update. It has not been re-tested on the SLRU+dedup design.
- Tie and contest tweaks are inert or harmful. Tie-admission (cycle 3), ghost exact-tie wins (cycle 7) and random admission jitter (cycle 7) either churn cold probation keys or never fire, because exact warm ties are rare with lifetime counts.
- Admission windows and window adaptation were negative in cycles 1, 2 and 6. Few keys return within a short recency trial, and a hill-climber drifts toward an LRU-like layout that this trace punishes.
- Victim-sampling width is exhausted. Deeper samples evict young residents, because the lowest unaged estimate marks the youngest key, not the coldest (cycles 2–3).
- SLRU-internal tweaks (demotion policy, segment ratio) move entries around without changing which keys get evicted, because the min-estimate sample already acts as a soft protected segment.
- Further sketch-accuracy work (conservative update on a wide sketch) is redundant. Width and conservative update fix the same error, so their gains don't add up.
- An evaluation gap has persisted in all 7 cycles: only hit-rate is scored. Per-op-time and metadata-size have never been measured, so memory and latency variants can at best tie and are often counted as losses.

## Open questions

- Is aging still negative on the current SLRU+dedup design? This is one decisive test. It must remove the ≤ 1 fast-path shortcuts first.
- Can a recency signal act where stale-high keys actually live (protected-segment demotion), or through a margin-based ghost bonus instead of an exact tie?
- Once metadata-size and per-op-time are scored, which combination dominates (narrow CU sketch, hash-cached fast path, SLRU), and what does SLRU's per-hit work cost?
- How narrow can the sketch get under SLRU? Width 8× looks like the floor. Could a recency tie-break between equal estimates push it lower?
- Is the current plateau simply workload-a's ceiling for TinyLFU-family policies?

## Per-policy notes

- **TinyLFU contest:** keep strict `>` with keep-victim-on-tie. The bottleneck was estimate noise, not the comparison rule.
- **Count-min sketch:** wide + dedup is the baseline. It is over-provisioned without SLRU (half width with u16 + conservative update was neutral). Under SLRU, narrowing costs a little, because probation-only sampling compares the newcomer against low-count entries where leftover collision error flips decisions.
- **W-TinyLFU window:** negative on workload-a in every form tried (fixed 1%, adaptive, and on top of SLRU).
- **SLRU (probation/protected):** a marginal gain over sampled LRU, then flat under every internal tweak. Protected keys are reachable only through demotion, so neither the contest nor the victim sample ever touches stale-high residents.
- **Sampled-victim LRU:** a small win at 8-way min-estimate. A wider sample hurts.
- **Ghost/recency history:** adds unpriced metadata and has had no effect so far through exact-tie use.
