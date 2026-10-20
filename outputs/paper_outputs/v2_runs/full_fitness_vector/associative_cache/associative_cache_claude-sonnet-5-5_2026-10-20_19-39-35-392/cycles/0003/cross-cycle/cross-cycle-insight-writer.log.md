# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-20T19:50:42+00:00

## timeline

### 19:50:56 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Scan-resistant insertion/eviction policy is the only lever that has moved hit rate (cycles 0002, 0003). Keeping one-hit-wonders from displacing reused entries pays off; capacity and associativity are not the limit.\n- Cheap, low-bookkeeping mechanisms rank best: stateless insertion position and the reference bit beat the frequency counter in 0002.\n- Intermediate insertion depth (len/4 to mid-bucket) with explicit promotion on hit keeps scan resistance while letting new entries prove reuse. This is the current policy baseline (0003 v003).\n- Latency tracks added per-access logic: fewer branches or less reordering means cheaper puts and a better get tail.\n\n## Recurring failures\n\n- Position-only LRU tweaks and geometry sweeps (wider buckets, partial promotion) gave flat or worse hit rate (0001). The cause is that conflict misses and promotion granularity are not the bottleneck.\n- Rotate/memmove micro-optimizations of the item move did not speed up get. Item movement does not dominate get time (0001, 0002, 0003).\n- Stacking scan-resistance mechanisms (reference bit plus low insertion) lost on both hit rate and speed (0003). They do not compound.\n- Pushing insertion all the way to the LRU end starves new entries of the chance to prove reuse.\n- Per-slot frequency counters cost latency on every hit and put for only a marginal gain.\n- Isolated latency gains with no mechanistic reason (put in 0001, one variant in 0002) were treated as noise or artifacts.\n\n## Open questions\n\n- Where does median get time go (hashing, bucket lookup, dispatch)? It has been unmeasured for three cycles and is the main untouched suspect.\n- Can mid-insertion keep its hit-rate gain at lower put cost, without shifting half the bucket?\n- Do the policy gains hold on workloads other than workload-a, or are they specific to its scan-heavy traffic?\n- Is the Winner label reliable for marginal variants (0003 v001 looked slightly worse than its parent)? Check the noise floor.\n- Is there any untested policy family (ghost/admission filter, adaptive insertion) that offers more than the saturated position axis?\n\n## Per-policy notes\n\n- Baseline LRU: insensitive to bucket width. Partial promotion hurts because the not-fully-promoted hit item becomes the next victim.\n- CLOCK reference bit: cheap and helpful alone (removes reordering). It is too coarse as a second chance when stacked on low insertion.\n- Frequency counter (2-bit with LRU tie-break): small hit gain, but it adds per-hit and per-put work. It is dominated by cheaper options.\n- Low-position insertion (LIP/BIP-style): clearest hit-rate gain alone. A rare hashed MRU escape added branch cost with weak evidence of benefit.\n- Mid-bucket insertion with transpose-by-one promotion: the current best policy baseline. Its cost is the median put shift.\n"
}
```

### 19:51:01 chat

I wrote `cross-cycle-insights.md` to the cycle 0003 cross-cycle output directory. It uses the four fixed sections in the required order and is built only from the three-cycle ledger digest.

- **Recurring wins:**
  - Scan-resistant insertion and eviction policy is the only lever that has moved hit rate.
  - Cheap mechanisms beat costlier ones.
  - Intermediate insertion depth with explicit promotion on hit is the current baseline.
- **Recurring failures:**
  - Geometry sweeps and position-only LRU tweaks.
  - Rotate/memmove micro-optimizations.
  - Stacking scan-resistance mechanisms.
  - Insertion all the way at the LRU end.
  - Per-slot frequency counters.
  - Unexplained latency gains, which the notes treat as noise.
- **Open questions:**
  - Where median get time goes. It has gone unmeasured for three cycles.
  - Whether mid-insertion can keep its hit-rate gain at lower put cost.
  - Whether the gains hold on workloads other than workload-a.
  - Whether the Winner label is reliable for marginal variants.
  - Whether any untested policy family is worth trying.
- **Per-policy notes:** baseline LRU, CLOCK reference bit, frequency counter, low-position insertion, and mid-bucket insertion with transpose-by-one promotion.

The file uses qualitative terms only and contains no invented numbers.


---

- finished: 2026-10-20T19:51:01+00:00
- duration: 18.73s
- status: ok
- input_tokens: 4
- output_tokens: 1661
- cache_read_tokens: 35900
- cache_write_tokens: 7608
- cost_usd: 0.051001
