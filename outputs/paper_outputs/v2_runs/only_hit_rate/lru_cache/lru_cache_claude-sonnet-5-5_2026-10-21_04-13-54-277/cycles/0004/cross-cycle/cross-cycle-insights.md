## Recurring wins

- Scan resistance is the lever that pays off: segmented LRU (v003 in cycle 1) and CLOCK-style second-chance (cycle 2) both gained hit-rate over strict LRU by keeping one-touch keys out of the main set.
- Growing the protected share of the segmented LRU gained hit-rate in two consecutive cycles, then flattened. Probation is now close to its minimum useful size.
- Each cycle's Winner came from a small, single-axis change to the current best, not from a combination of ideas.

## Recurring failures

- Weakening the protected segment (a smaller share, or direct ghost-to-protected admission) lost hit-rate.
- Refinements that sit on rarely-taken paths matched their parent exactly and gave no signal: the probation-MRU ghost list and the protected-eviction second-chance bit.
- Approximate-recency shortcuts (stamp-based relink skipping) gave up a little hit-rate. Their possible speed benefit was never measured.
- Strict-LRU micro-optimizations were neutral on hit-rate, so they cannot win while hit-rate is the only measured axis.
- Single-constant tuning is exhausted. Its returns shrank each cycle.

## Open questions

- Metadata-size and per-op-time have been missing from every fitness vector for four cycles. Is this a harness fault? It blocks judging every compactness or speed claim, and selection runs on hit-rate alone.
- Every result comes from one scan-heavy trace at one cache size. Do the gains hold on other traces and sizes, especially ones with long-distance reuse or little scan pressure?
- Would adaptive segment sizing (ARC-style) beat any fixed share?
- Can segmented LRU combine with CLOCK or frequency ideas? Frequency bits looked incremental over one-bit CLOCK.
- Does the relink-skip shortcut or CLOCK's cheaper hit path pay off in time? It cannot be known until timing is measured.

## Per-policy notes

- Segmented LRU: the best-supported line so far. A high protected share (about 90–95%) is near its plateau. Ghost-list variants have not helped. Protected-eviction tweaks are not worth more effort. The next gains should come from probation or promotion logic, or from adaptive sizing.
- CLOCK (one-bit second chance): a small hit-rate gain over strict LRU through a different mechanism. Its claimed size and speed advantages are unverified.
- Frequency-aware CLOCK: a modest gain, no win. Adds little over one-bit CLOCK on this trace.
- Strict LRU (baseline): the compact-layout and fast-path ideas stay untested. Do not revisit them until size and time are measured.
