# Cycle 0006 Insight

## Cohort outcome

All three variants branched off cycle 0005's dual-source-ghost incumbent
and, for the first time, attacked latency through **scan mechanics
themselves** rather than tuning ghost length or content. The two
median-latency wins landed — an O(1) ghost membership index and a
resident scan reordered to hit main first — while one more ghost-length
widening confirmed the 50% knee still holds and bought nothing. All three
were Winners, but the productive moves were both pure latency plays with
hit-rate held bit-for-bit constant.

## Patterns

- The hit-rate frontier is now treated as closed: every change this cycle
  was explicitly hit-rate-neutral (identical key sets) or a failed
  hit-rate probe. The last ghost-length step (1/2 → 5/8) recovered no
  near-reuse hits even under the doubled dual-source inflow, retiring the
  "re-find the knee for the new content regime" thread — content, not
  length, was the binding constraint, and content is now also tapped out.
- Latency is decomposable into distinct scans, and each yields to a
  *structural* fix rather than a length trim. The ghost membership walk in
  put falls to an O(1) container-level hash index; the resident walk's get
  p50 floor — stuck for many cycles — finally moves once main is scanned
  before small. Both confirm the long-suspected scan drivers by eliminating
  them.
- Every median win redistributes cost to the tail. The hash index trades a
  small predictable per-element scan for rarer hashing/rehash spikes (worse
  put p99); the scan reorder pushes the rarer small-resident, miss, and put
  lookup paths to the tail. p50/p99 are now pulling apart on the same knob.
- Metadata discipline held in spirit but stretched: the scored `Entry` size
  stayed 24 bytes, yet the O(1) index grows the (unscored) container — a new
  category of cost that sits outside the scored summary.

## Open questions

- Are the median latency wins net-positive once the tail regressions
  (put p99 hashing spikes, reordered p99 walks) are weighed — or does the
  scored summary reward p50 enough that the tail cost is invisible?
- Is there a structure that gives O(1) membership *without* a parallel hash
  set's tail spikes (e.g. reusing the resident index, or a cheaper probe)?
- With hit-rate closed and the two main scans addressed, what scan or path
  is left to shorten — is workload-a now at both a hit-rate and a
  median-latency floor?

## Continuation

Treat the ghost frontier — length **and** content — as fully closed: 50%
is confirmed terminal and no widening recovers hits, so stop re-probing it.
The live axis is now **latency via structure, not tuning**: the O(1) index
and the main-first scan reorder both moved long-stuck floors, so pursue
further scan-structural wins. But guard the tail explicitly — both winners
shifted cost into p99, so the next step should look for an O(1) membership
that avoids the hash-resize spike, and weigh whether reorder/index wins
survive a p99 reckoning. Keep the scored `Entry` at 24 bytes, but note that
unscored container growth is now in play. Expect hit-rate to stay flat —
this workload is hit-rate-bound at this capacity, and the remaining game is
how cheaply the same hits are served.
