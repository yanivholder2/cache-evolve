# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-20T11:33:57+00:00

## timeline

### 11:34:09 thinking



### 11:34:13 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Re-reference signals beat recency-only ordering. Keeping keys that have been hit again protects them from bursts of one-shot inserts, which is the pollution pattern the bucket-local work keeps targeting. Seen with frequency-based victim selection (cycle 2) and second-chance reference bits (cycle 3), which use different state for the same effect.\n- The second-chance variant is the first result to improve hit-rate and put cost together, so it is the current base to build on.\n\n## Recurring failures\n\n- Moving items within a bucket (midpoint promotion, midpoint reinsertion) lost hit-rate on workload-a. Do not revisit as specified.\n- Bucket-local ghost memory has not paid off. A short ring (cycle 1) and a longer ring (cycle 2) both failed to lift hit-rate, so the reinsertion idea looks weak, not just under-sized. Do not tune ring length further.\n- Extra lookup layers add cost without hit-rate gain. The fingerprint prefilter on 32-slot buckets added a second scan on get and write work on every put, with no hit-rate change.\n- Put-side victim-search shortcuts have not worked. A cached minimum is invalidated by get-hit counter updates. A merged scan with per-eviction halving does not help. Extra miss-driven decay erased frequency protection too fast. Periodic halving is the working aging schedule and should stay.\n\n## Open questions\n\n- Does re-reference retention hold on traces other than workload-a? Every winning signal so far has only been measured there.\n- Is the second-chance gain driven by the reference bit itself? The overlay was unreadable in cycle 3, so the attribution rests on the hypothesis and metadata delta, not on code.\n- Can per-item state shrink to one bit without losing the hit-rate gain? Can the get-side write on hits be avoided, for example by setting reference state only on some hits?\n- Can the frequency or reference victim search get cheaper, for example by considering fewer ways, without losing hit-rate?\n- Does the small wider-bucket hit-rate gain (cycle 1, single cycle) hold on other traces? It is the only layout result that improved hit-rate and is worth re-measuring before building on.\n- Put and get latency shifts have been single-run and partly unexplained by the diffs. Treat any timing gain as uncertain until re-measured.\n- The metadata-size probe has not reported any change in any variant, including ones that clearly add per-bucket or per-slot state. Storage cost is unmeasured until the probe counts that state.\n\n## Per-policy notes\n\n- Bucket width / geometry layout: the only hit-rate gain seen was small, and the longer linear scan on get was the cost. Re-measure before building on it.\n- Midpoint promotion and reinsertion: negative on hit-rate and add shifting work on the hot path. Drop as currently specified.\n- Bucket-local ghost memory: no gain at short or longer ring length. Deprioritize.\n- Fingerprint prefilter: no hit-rate change, extra get scan and put writes. Not worth carrying forward.\n- Frequency-based victim selection (per-slot access counters, periodic halving): retains re-referenced keys through one-shot bursts. Its put-side full-bucket scan is the open cost. Keep the periodic halving schedule. Cached-minimum and merged-halving variants failed.\n- Second-chance reference bit: hit-rate up and put cost down, with slightly higher get cost from a per-hit write and metadata growth beyond one bit. Next steps: confirm the mechanism, reduce per-item state toward one bit, and cut the get-side write.\n"
}
```

### 11:34:16 chat

I wrote `cross-cycle-insights.md` to the cycle 0003 cross-cycle output directory, with the four fixed sections in order. It is built only from the inlined ledger digest. I did not read the previous insights file, and I did not use any numbers.

The main points:

- **Recurring win:** re-reference signals beat recency-only ordering. Frequency-based victim selection (cycle 2) and second-chance reference bits (cycle 3) both keep re-referenced keys through bursts of one-shot inserts.
- **Recurring failures:** midpoint promotion and reinsertion, bucket-local ghost memory at both ring lengths, the fingerprint prefilter, and the put-side victim-search shortcuts.
- **Open questions:** whether the wins hold beyond workload-a, whether the second-chance gain is really mechanism-driven (its overlay was unreadable), how far the per-item state can shrink, and why the metadata-size probe reports no change.
- **Per-policy notes:** one entry per policy family, with the next step for each.


---

- finished: 2026-10-20T11:34:16+00:00
- duration: 19.16s
- status: ok
- input_tokens: 4
- output_tokens: 3839
- cache_read_tokens: 36012
- cache_write_tokens: 10052
- cost_usd: 0.00429
