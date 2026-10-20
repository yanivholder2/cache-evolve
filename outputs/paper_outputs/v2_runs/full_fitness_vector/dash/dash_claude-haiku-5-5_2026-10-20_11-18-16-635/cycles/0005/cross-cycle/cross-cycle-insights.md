## Recurring wins

- Protecting hot residents at eviction is the most consistent hit-rate lever. Frequency-aware victim selection gained hit rate in cycle 0002 (saturating counter) and cycle 0004 (full-scan lowest-counter), and the signal is consistent with the protect-at-eviction direction in cycles 0002 and 0003.
- Deferring stash-to-target promotion keeps its hit-rate gain when the first-hit bookkeeping stays cheap (cycle 0003, then cycle 0004). Its cost sits in the get tail, not in the deferral itself.
- Keeping single-touch entries out of target buckets gave a small hit-rate gain (cycle 0003), consistent with the concern that early entries displace hot target residents.

## Recurring failures

- Gating insert position on bucket fullness or MRU-style promotion trades hit rate for latency. Front-of-full-bucket inserts cost hit rate (cycle 0002), and plain MRU promotion protects the promoted key only at the cost of displacing hot residents (cycle 0001).
- Blocking first-touch admission lost hit rate on workload-a (cycle 0003), so early admission matters on that trace.
- Stash occupancy fast paths raised get and put latency rather than lowering it (cycle 0002).
- Recency tie-breaking among equal eviction counters moved no hit rate and added eviction-path cost (cycle 0003).
- Narrowing the frequency-aware search loses its gain. Two-choice sampling (cycle 0005 v001) and stash-only restriction (cycle 0005 v002) both fell below the full-scan version, and stacking the counter on stash victims hurt hit rate and tails.
- Record repacking does not move eviction. It was hit-rate neutral (cycle 0004), and in cycle 0005 it did not shrink the segment it was meant to shrink.
- Per-op latency claims have not held up in any cycle. Put and get movements are attributed to changes whose code paths do not explain them, and Winner status has not tracked these measurements. Do not credit latency gains from this run until they are re-measured.

## Open questions

- Does the full-scan lowest-counter gain (cycle 0004) hold on a second, non-workload-a trace? Every hit-rate signal so far comes from workload-a, and the stash-dominant and scan-heavy traces requested since cycle 0003 have not been run.
- How much of the put-side cost of the full-scan victim search is the scan itself? Cycle 0005 showed that shortening the scan did not make puts cheaper, so the source is still unidentified. Is there a cheaper way to keep the true minimum, such as maintaining it incrementally?
- Is v001's hit-rate loss from cycle 0001 (MRU insert) stable across traces, and does it come only from full buckets?
- Does the deferred promotion keep its hit-rate gain on a stash-dominant trace?
- Why are variants with recorded hit-rate or per-op regressions labelled Winners? The Winner basis is not visible in any cycle's insight, and this has been unresolved since cycle 0001.
- Does the admission gate lose hit rate on scan-heavy traces, where it was meant to help?

## Per-policy notes

- Frequency-counter victim selection: the only policy with a repeated hit-rate gain. It needs the full bucket scan to hold the gain. Its put-path cost is unresolved, and it has not been tested beyond workload-a.
- Stash-to-target promotion (deferred): keeps hit-rate gain in cycles 0003 and 0004 when the first-hit state is cheap. Stacking frequency counters on its stash victims is not safe (cycle 0005).
- Recency (LRU/MRU-style) tie-breaking: no hit-rate effect on the measured trace, and it adds cost. Not worth building on.
- Insert position (MRU, back-of-queue, fullness-gated front insert): trades hit rate for latency, and the trade is worse when buckets are full. Keep the back-of-queue protection for promotions into full buckets.
- Admission gating on first touch: lost hit rate on workload-a. Revisit only if aimed at scan-heavy traces.
- Layout (record repack): policy-neutral. It does not change eviction, and its per-op gains are unexplained.
