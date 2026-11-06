## Recurring wins

- Cold insertion (placing new items below re-referenced ones) has produced the Winner in both cycles and stays the main source of hit-rate gains on the MSR traces.
- Coarse or frequency-aware recency (CLOCK, then GCLOCK) has not cost hit-rate in either cycle. GCLOCK improved on its CLOCK parent in every trace×size cell. Exact LRU ordering does not seem to be what drives hit-rate.
- Lengthening probation (a deeper or absolute floor, occasional MRU insertion) recovers part of the hits lost in small buckets, and the recovery is consistently larger on hm_0 than on web_0.

## Recurring failures

- Any rule that lets new items outrank reused ones hurts. Bounded promotion with MRU insertion lost hits in cycle 1. BIP-style MRU insertion lost hits on web_0 at the large size in cycle 2.
- A single fixed rule (one fraction, one period) can't serve both trace types and both bucket sizes. Each fixed tweak so far helped one trace×size cell and left another flat or worse.
- Cost has never been measured: no per-op-time or metadata-size data in either cycle. Every latency or overhead claim is still untested (mid-vector inserts, CLOCK/GCLOCK sweeps, candidate scans).

## Open questions

- Can adaptive insertion fix the trace split, for example set dueling between LIP and BIP, or a probation depth or MRU-insert rate that scales with bucket size?
- Do the absolute probation floor and occasional MRU insertion stack, or do they recover the same reuse?
- Can the GCLOCK lineage add a cold-insertion analogue (new items starting below referenced ones) and combine the strengths of both lineages?
- Does the CLOCK/GCLOCK line actually lower latency, and does its hit-rate edge hold once worst-case sweep cost is measured?

## Per-policy notes

- LRU with cold insertion (LIP-like): the strongest lineage, with the most gain in large (1024-slot) buckets. Fixed-fraction probation is too short in small (128-slot) buckets, and that cuts off real reuse, mostly on hm_0.
- BIP-style (occasional MRU insertion): small net gain, but it splits by trace. It helps hm_0 in small buckets and hurts web_0, whose cold traffic is scan-like. Further tuning of the fixed period alone is not worth pursuing.
- Bounded or graded promotion: not useful on its own while new items still enter at MRU. It may be worth revisiting only on top of cold insertion.
- CLOCK / GCLOCK: hit-rate matches or beats LRU, and the edge grows with bucket size (strongest on hm_0 at the large size). It does no element shifting. It is a credible parallel branch, pending cost measurement.
- Trace behavior: hm_0 has reuse beyond short probation windows and rewards longer retention of new items. web_0 is scan-dominated and rewards draining cold items aggressively.
