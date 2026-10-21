# Cycle 0009 Insight

## Cohort outcome

Three small refinements were tried: tighter ghost-table admission that skips cold victims and caps the restored counter (v001), dropping an incoming key when its whole bucket is hot (v002), and a compact one-bit reuse flag with a second-sighting bit-array admission filter (v003). v002 won, but its hit-rate gain over the parent was within noise. v001 slipped slightly and v003 gave only a marginal gain over baseline LRU.

## Patterns

- This is the fourth cycle in a row with noise-level gains. The Winner label here is probably selection noise, not a real improvement.
- Ghost-history tuning (v001) went the wrong way. Remembering cold victims appears to help, so filtering ghost entries by heat removes useful signal.
- Bucket-level scan resistance (v002) did almost nothing. All-hot buckets are likely rare on workload-a, or the counter and the second-sighting filter already protect hot entries.
- v003 shows a compact design reaches only a marginal gain. Its stated goals, metadata size and per-op time, were never measured, so its value is untested.
- The same gap remains as before. Only hit-rate on one trace and one size is reported, per-op-time and metadata-size are still missing from the fitness vector, and the authors could not read the overlay sources.

## Open questions

- Is any hit-rate headroom left on workload-a, or is every lever at the noise floor?
- Would the compact v003 design be worth adopting if its cost savings were measured?
- Do scan-heavy or other workloads show a benefit from bucket-level protection (v002) that this trace hides?

## Continuation

Treat v002 as a marginal incumbent at best, and drop further ghost-table tuning (v001) and bucket-level admission blocking. Stop refining admission, placement, eviction state and ghost history on workload-a. Next cycle should first add per-op-time and metadata-size to the fitness vector, and validate the incumbent and v003's compact design on other workloads and cache sizes. If it explores new levers, choose ones outside those areas, such as adaptive sizing or scan-resistance for different access patterns.
