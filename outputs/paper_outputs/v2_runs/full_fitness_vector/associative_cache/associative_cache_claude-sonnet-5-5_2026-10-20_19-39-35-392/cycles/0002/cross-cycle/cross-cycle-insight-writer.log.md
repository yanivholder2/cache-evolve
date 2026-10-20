# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-10-20T19:46:57+00:00

## timeline

### 19:47:06 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Scan-resistant policy changes (CLOCK reference bit, low-position/LIP-BIP insertion) are the only thing that has moved hit rate so far. They stop one-hit-wonders from displacing reused entries.\n- Cheap, stateless or near-stateless mechanisms beat heavier ones. Gain ranked inversely with bookkeeping cost.\n- Latency tracks bookkeeping cost. Removing reordering (reference bit) helped put and the get tail. Adding per-hit work hurt.\n\n## Recurring failures\n\n- Position-only tweaks to LRU and geometry (wider buckets, partial promotion) gave no or negative hit-rate change. Associativity and capacity do not appear to be the limit on workload-a.\n- Item-movement micro-optimizations (rotate vs remove+push) did not speed up get. Item movement is not what dominates get time.\n- Per-slot frequency counters give marginal hit gain and add latency on every hit and put.\n- Unexplained latency gains (put in cycle 0001, one variant's latency in cycle 0002) have shown up on variants that did not touch that path. Treat them as noise or artifacts until they reproduce.\n\n## Open questions\n\n- Do the scan-resistant gains stack (low-position insertion plus a reference bit)?\n- Where does median get time go (hashing, bucket lookup, dispatch)? No cycle has measured it, and the lookup path is the main remaining suspect.\n- Do the policy gains generalize beyond workload-a, or are they specific to its scan-heavy traffic?\n- Is frequency information redundant once scan resistance is in place?\n- Would ghost-list or admission-filter policies add anything over the cheap mechanisms?\n\n## Per-policy notes\n\n- LRU baseline: insensitive to bucket width. Partial promotion hurts hit rate because a hit item that is not fully promoted becomes the next victim.\n- CLOCK reference bit: wins on hit rate and makes put and the get tail cheaper by removing reordering.\n- Low-position insertion (LIP/BIP-style): clearest hit-rate gain, with no extra state.\n- Frequency counter (2-bit, LRU tie-break): small hit gain, but slower put and get. Lowest priority of the three.\n- Metadata size has not grown in any variant so far.\n"
}
```

### 19:47:11 chat

I wrote `cross-cycle-insights.md` to the cycle 0002 cross-cycle output dir, using the four fixed sections in order. It's built only from the two ledger digests, with no numbers added.

- **Recurring wins:** Scan-resistant policy changes (the CLOCK reference bit and low-position insertion) are the only approaches that have raised hit rate. Cheap mechanisms beat heavier ones, and latency tracked how much bookkeeping each variant added.
- **Recurring failures:** Wider buckets, partial promotion and rotate-based moves gave no gain. The frequency counter gave a small hit-rate gain but slowed puts and gets. Latency gains with no mechanism behind them are noted as likely noise.
- **Open questions:**
  - Do low-position insertion and the reference bit stack?
  - Where does median get time go?
  - Do the gains hold beyond workload-a?
  - Is frequency information redundant once scan resistance is in place?
  - Would ghost-list or admission-filter policies add anything?
- **Per-policy notes:** These cover baseline LRU, the CLOCK reference bit, low-position insertion and the frequency counter. They also note that no variant has grown metadata.


---

- finished: 2026-10-20T19:47:12+00:00
- duration: 14.18s
- status: ok
- input_tokens: 4
- output_tokens: 1320
- cache_read_tokens: 35132
- cache_write_tokens: 6590
- cost_usd: 0.043364
