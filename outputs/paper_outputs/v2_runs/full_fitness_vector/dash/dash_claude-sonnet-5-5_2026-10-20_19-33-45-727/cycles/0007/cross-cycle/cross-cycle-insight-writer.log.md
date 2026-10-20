# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:09:55+00:00

## timeline

### 20:10:05 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- The second-hit admission filter is the one real policy win. It was found in cycle 0001, bit-packed to baseline item size in cycle 0002, and independently re-derived from baseline in cycle 0004, where it improved both hit-rate and latency.\n- Packing flags into spare bits keeps metadata flat. Footprint is no longer a constraint for counters or ref bits.\n- Scan resistance (keeping one-touch items out of the hot tiers) is the lever that works. Admission gating beats insert-position tuning.\n- A doorkeeper that declines first touches saves put-side work (insert and eviction). This is the only credible latency mechanism seen, but it costs hit-rate when the ring is small.\n\n## Recurring failures\n\n- Insert-position-only variants and stacked position/admission rules overlap and do not add. They gave small or mixed gains and are exhausted.\n- Richer admission or recency signals (multi-bit counter, ref bit, one-slot ghost) added almost no hit-rate. The second-hit filter already catches most of what they target.\n- Hot-path edits to the incumbent regressed latency in roughly six consecutive cycles. These included guards, shortcuts, lookup atomics, free-slot fast paths, promotion skips, hoisting, tag-first scans, reader-writer locks and dead-store deletion. Extra branches or work cost more than they save.\n- Regressions often appeared on paths the change did not touch, so layout effects or noise are likely. The noise floor has never been measured.\n- Subtractive cleanup ran out of room. The cycle 0004 gain probably came from the filter, not from dead-code removal.\n- Winner status is weak evidence. Flat hit-rate and metadata qualify a variant even when latency regresses.\n- Variant authors could not read the source, so the mechanisms offered are inferred.\n\n## Open questions\n\n- What is the run-to-run noise floor for get and put latency on the unchanged incumbent? This has been asked for four or more cycles and is still unanswered.\n- Where does get and put time actually go (lock, memory access, bucket scan)? Nothing has profiled it.\n- Does a second trace (scans, frequency skew) change any conclusion? Everything so far uses the recency-skewed LIRS trace.\n- Can a cache-line-sized, aged doorkeeper keep the larger doorkeeper's hit-rate and the small ring's put savings?\n- Does the cleaned-up minimal filter match or beat the cycle 0002 bit-packed incumbent?\n\n## Per-policy notes\n\n- DASH second-hit bit-packed filter: this is the incumbent and base. It has baseline item size, and hit-rate looks saturated on this trace.\n- DASH doorkeeper ring (small fixed): large put-side latency drop but lost hit-rate because recurring keys age out before their second touch.\n- DASH doorkeeper (larger, two-hash, aged): recovers hit-rate but costs put latency through extra probes, footprint and resets.\n- DASH position variants (uniform MRU, cold/hot split, cold demotion): modest early hit-rate gains, with trade-offs against latency. Drop them.\n- Ghost history (one slot): near-free in metadata but almost never matches. A larger ghost is untested and of doubtful value.\n- Next step: measure the noise floor and profile, or use a second trace, before crediting any latency change.\n"
}
```

### 20:10:09 chat

I wrote `cross-cycle-insights.md` in the cycle 0007 `cross-cycle` output directory. It has the four required sections in order, built only from the cycle 0001–0007 ledger digest.

- **Recurring wins:** the bit-packed second-hit admission filter, which was re-derived from baseline in cycle 0004 and confirmed as the one real policy win. The notes also say flags packed into spare bits keep metadata flat. A doorkeeper that declines first touches is the only credible latency mechanism seen.
- **Recurring failures:** position-only and stacked variants overlap and are exhausted. Richer admission signals added almost nothing. Hot-path edits to the incumbent regressed latency for about six cycles running. Winner status is weak evidence.
- **Open questions:**
  - The run-to-run noise floor has gone unmeasured for four or more cycles.
  - Profiling and a second trace are both still undone.
  - Whether a cache-line-sized aged doorkeeper can keep both hit-rate and put-side savings.
- **Per-policy notes:** grouped by variant family: second-hit filter, small and large doorkeepers, position variants, and ghost history.

I kept the numbers qualitative and did not read the previous insights file.


---

- finished: 2026-10-20T20:10:09+00:00
- duration: 14.32s
- status: ok
- input_tokens: 4
- output_tokens: 1678
- cache_read_tokens: 38917
- cache_write_tokens: 10720
- cost_usd: 0.064223
