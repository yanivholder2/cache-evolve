# Cycle 0005 Insight

## Cohort outcome

Three variants targeted probation on the 95%-protected segmented LRU: a TinyLFU-style sketch admission gate, a change to where demoted protected nodes land in probation, and a frequency-aware probation eviction scan. Only the admission gate (v001) won, with a marginal hit-rate gain. The other two matched their parent exactly.

## Patterns

- Probation-side tweaks are nearly inert at this protected share. Demotions are rare, so where demoted keys land in probation changes nothing.
- The frequency-scan variant was dead by construction. Frequency was reset or never persisted in the segment being scanned, so every candidate tied and the scan fell back to plain LRU order.
- Admission filtering gave a small gain, but segmentation already keeps one-hit scan keys in the small probation segment. The sketch's extra work (hashing, counters, periodic decay) buys little. Counting rejected misses lets them be admitted on retry, and the gate can reject new hot keys.
- Hit-rate gains remain marginal and rest on one scan-heavy trace at one cache size.
- Metadata-size and per-op-time were not used to judge anything this cycle, and the cost of the sketch and the extra freq byte per node is unverified.

## Open questions

- Is the sketch gate worth its time and memory cost, if per-op-time and metadata-size could be measured?
- Does admission filtering matter more on traces with long-distance reuse or heavier scan pressure?
- Would frequency help if it persisted across demotion, or covered protected entries?
- Is the measurement gap on metadata-size and per-op-time (now five cycles) a harness fault?

## Continuation

Stop varying probation placement and probation-only eviction scans; the segmented design is saturated on this trace. Keep the admission gate only if cost can be measured, and prefer cheaper variants such as not counting rejected misses or a tiny doorkeeper. Before more hypotheses, fix the cost measurement and add a second trace or cache size. Then try ideas that change a different axis, such as adaptive sizing, CLOCK hybrids, or frequency that persists across segments.
