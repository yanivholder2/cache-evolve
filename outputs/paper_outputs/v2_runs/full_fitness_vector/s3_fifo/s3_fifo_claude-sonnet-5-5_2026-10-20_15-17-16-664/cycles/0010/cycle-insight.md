# Cycle 0010 Insight

## Cohort outcome

The cycle tried three small refinements: zeroing frequency on small-to-main promotion (v001), age-gating ghost re-admission to the newer half of the ghost ring (v002), and reusing one computed hash across the resident, ghost and insert probes in `put` (v003). None has a mechanism-backed gain. All three were selected as Winners, but v001 is neutral, v002 gained only a tiny amount of hit rate at a latency cost, and v003 hurt.

## Patterns

- All three Winners are probably noise-assisted. This continues the trend since Cycle 0008 that selection without a mechanism is unreliable.
- Two variants targeted main-admission quality (freq reset, stale-ghost gating). Both showed that weak main entries are not a significant source of loss on this trace. Hit rate barely moved.
- Both latency-oriented or bookkeeping changes (v002's sequence counter, v003's hash carrying) added per-entry state and slowed things, mostly in the `put` tail. This repeats the Cycle 0009 lesson that added state costs more than it returns here.
- v003 shows hashing small integer keys is not the `put` tail cost. That closes the hashing line, which Cycle 0008 had already called exhausted.
- Analyses for all three variants were written without reading the sources, so the mechanisms are inferred and weaker than usual.
- Evidence is still one trace at one cache size.

## Open questions

- Is the trace simply near its ceiling for this policy family, so that small policy tweaks cannot show up?
- Is the `get` slowdown seen in v002 real, or just run-to-run variance on this harness?
- Do any of the earlier latency gains (tagged index, split ghost map, keyed lookup) survive repeated runs?

## Continuation

Drop freq-reset, ghost age-gating, and hash reuse. Do not add per-entry state or counters. Stop micro-tweaks to admission and hashing. The next cycle should first establish a repeated-run noise floor and test on other traces and cache sizes, with an explicit heap measurement, then pick only hypotheses with a clear mechanism, such as a structural change to promotion or ghost policy that can move hit rate on a different workload shape.
