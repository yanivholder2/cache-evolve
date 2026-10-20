# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:07:09+00:00

## timeline

### 20:07:20 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Cheap scan resistance moves hit rate: policies that stop one-hit-wonders from displacing reused entries (low or mid-position insertion, reference bit alone) paid off from cycle 0002 onward. Capacity and associativity were not the limit.\n- Mid-bucket (about len/4) insertion with one-step transpose promotion is stateless and reproducible (cycles 0003, 0006). Swap-based insertion (cycle 0004) cut its put cost with no hit-rate change.\n- Put-path cost is the lever that moves: removing shifts, redundant hashing and modulo cut put latency. Exact order among displaced entries doesn't matter; where the new entry lands does.\n- Gradual one-step promotion works as a frequency filter on the scan-heavy trace.\n\n## Recurring failures\n\n- Get latency has had no real gain in seven cycles. Item movement, arithmetic, hashing, tag-first layout and alignment are all ruled out. Small get shifts are noise or codegen effects.\n- Position-only tuning is saturated. Wider buckets, len/3, len/5 and LRU-end insertion all failed to help, so len/4 is a local optimum.\n- Stacking scan-resistance mechanisms (reference bit plus low insertion) does not compound. It loses on both hit rate and speed.\n- Per-slot frequency counters and one-entry ghost filters add per-access work for little or negative hit-rate gain.\n- Stateless randomization (hash-parity victim, hash-conditional escapes) carries no reuse signal. It adds branches and noise.\n- Aggressive promotion (geometric jump) removes the frequency filter and hurts hit rate.\n- Put-hit control-flow tweaks are worthless because puts are mostly misses.\n- Allocation-only layout tweaks (alignment) did not reproduce their put-tail gain.\n- Put-latency gains with no mechanism have repeatedly turned out to be noise.\n- Winner labels are unreliable. For five or more cycles, Winners included variants whose own insight reports a loss. Weigh the mechanism and the insight text over the label.\n\n## Open questions\n\n- Which Winners are real? A noise check (repeat runs) is needed.\n- Where does get time go? Harness, dispatch, or the key/value path is untested.\n- Can a small real reuse signal fit in the 40-byte metadata, for example a jump only after a second hit, or a deeper selective admission history?\n- Do any gains hold on a second workload, or are they specific to workload-a's scan-heavy traffic?\n- Can the put-miss and insertion path be made cheaper still?\n\n## Per-policy notes\n\n- LRU baseline (move-to-front): position and rotate tweaks gave nothing. Hit-rate gains need a scan-resistance signal.\n- CLOCK reference bit: cheap and helpful alone (it removes reordering). It hurts when stacked on low insertion, because a one-bit second chance is too coarse.\n- Frequency counter (2-bit, LRU tie-break): marginal hit-rate gain, with latency cost on every hit and put. Dropped.\n- Low or mid insertion (LIP/BIP-style): the clearest gain. Too deep (LRU end) starves new entries; len/4 is the sweet spot. Swap-based insertion with transpose promotion is the current baseline.\n- Ghost admission filter: one entry per bucket is too weak and slightly hurts hit rate.\n- Hash-based eviction and geometric promotion: both lose hit rate. Don't revisit.\n"
}
```

### 20:07:33 chat

I wrote `cross-cycle-insights.md` in the output dir. It has the four fixed sections in order and is built only from the cycle 0001–0007 ledger digest.

Main takeaways it records:
- **Recurring wins:** cheap scan resistance moves hit rate, and mid-bucket insertion at about len/4 with transpose promotion is the stable baseline. Cutting put-path cost works too.
- **Recurring failures:** get latency has not improved in seven cycles, and position-only tuning is exhausted. Stacking scan-resistance mechanisms doesn't compound. Winner labels often contradict the variants' own insights.
- **Open questions:** whether any Winner is real, where get time actually goes, whether a small reuse signal fits in the 40-byte metadata, and whether gains hold on a second workload.
- **Per-policy notes:** these are grouped as LRU baseline, CLOCK reference bit, frequency counter, low/mid insertion, ghost filter, and hash-based eviction with geometric promotion.

I used qualitative wording only and invented no numbers.


---

- finished: 2026-10-20T20:07:33+00:00
- duration: 24.22s
- status: ok
- input_tokens: 4
- output_tokens: 1682
- cache_read_tokens: 38976
- cache_write_tokens: 10843
- cost_usd: 0.064766
