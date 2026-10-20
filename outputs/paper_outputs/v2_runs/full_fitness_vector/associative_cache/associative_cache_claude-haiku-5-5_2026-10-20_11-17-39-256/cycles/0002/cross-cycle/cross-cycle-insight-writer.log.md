# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-10-20T11:29:05+00:00

## timeline

### 11:29:11 thinking



### 11:29:15 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Retention of re-referenced keys is the only direction that has improved hit-rate. Wider buckets (cycle 1) and frequency-based victim selection (cycle 2) both helped by keeping reused items against one-shot inserts. The gains are small so far.\n- The frequency signal beat recency-only ordering where recency had not helped. This is the strongest hit-rate result in the ledger.\n\n## Recurring failures\n\n- Moving items within a bucket on hit or reinsertion (midpoint promotion, midpoint reinsertion) lowered hit-rate on workload-a and added shifting work to the hot path.\n- Bucket-local ghost memory has not lifted hit-rate in either form tried: a small ring (cycle 1) and a longer ring (cycle 2). The reinsertion idea looks weak here, not just undersized.\n- Extra linear scans on the get path cost time without hit-rate gain. This came from a fingerprint prefilter (cycle 2) and from wider buckets' longer scan (cycle 1).\n- Per-op latency regressions have followed each structural change. Put-side full-bucket victim scans are the current source.\n- Metadata-size has not moved in any variant, including ones that add per-bucket or per-slot state. The probe is not measuring new state and cannot be used to judge storage cost.\n\n## Open questions\n\n- Do the hit-rate gains from wider buckets and frequency victim selection hold on traces other than workload-a, or are they tied to its one-shot insert pattern?\n- Can frequency victim selection be made cheap (running minimum, fewer ways) without losing its hit-rate gain?\n- What should the size probe count so per-bucket and per-slot state shows up?\n- Put and get latency shifts have been single-run and sometimes unexplained by the diffs. Are they noise or real layout effects?\n\n## Per-policy notes\n\n- Frequency-based victim selection: the only hit-rate winner so far. Its put-side full-bucket scan is the main cost. Build on it and cut that cost first.\n- Wider buckets (geometry layout): small hit-rate gain with a longer get-path scan. Re-measure on more traces before building on it.\n- Midpoint promotion and midpoint reinsertion (recency-position policies): hurt hit-rate and added hot-path shifting. Dropped as specified.\n- Bucket-local ghost memory: no hit-rate lift at either ring length. Do not tune ring length further.\n- Fingerprint prefilter (lookup filter, not an eviction policy): no hit-rate change, added get and put work. Dropped.\n"
}
```

### 11:29:17 chat

I wrote `cross-cycle-insights.md` in the cycle 0002 cross-cycle output dir, with the four required sections in order. I did not read the previous version of the file.

The file is short because the ledger is thin: the only cross-cycle signal that repeats is that the two hit-rate gains (wider buckets and frequency-based victim selection) both came from keeping re-referenced keys. The gains are small and so far seen only on workload-a. Most other points are single-cycle observations that recur, such as the get and put cost of linear scans and the size probe not counting new state.


---

- finished: 2026-10-20T11:29:17+00:00
- duration: 12.13s
- status: ok
- input_tokens: 4
- output_tokens: 2010
- cache_read_tokens: 35096
- cache_write_tokens: 7469
- cost_usd: 0.00285
