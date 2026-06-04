# Cross-cycle insights

## Recurring wins

- Frequency / reuse signals ride an existing per-item counter, so
  retention and eviction changes that use them stay metadata-flat
  (cycles 2 and 3). Reuse-based logic is the cheap path on the metadata
  axis.
- The Pareto front keeps advancing on latency and metadata, and those
  gains have come from structural promotion-path edits (e.g. an O(1)
  tail push replacing a bucket shift). Single-lever orthogonal bets
  reliably show which axis is cheapest to move.
- Eviction-victim selection surfaced as a genuinely independent lever —
  changing *who leaves* without touching promotion/position — and held
  the front, confirming a clean new axis distinct from every prior
  cycle's promotion work.

## Recurring failures

- Pure-recency promotion (biasing toward the MRU end) is actively
  penalized on the scan/loop trace: it keeps soon-cold keys resident and
  costs hits. Recency is the wrong retention signal for this workload.
- Reuse/frequency rules implemented with an O(n) bucket or victim scan
  keep taxing get latency (cycles 2 and 3). The requested cheap O(1)
  variants have not materialized on the front.
- The cheap reuse-gate / second-hit admission promotion refinement has
  been requested two cycles running and failed to surface a comparable
  result both times — it is either not being realized or not emitting
  comparable output.
- Hit-rate-targeted bets remain unjudgeable: the scan/loop trace is
  absent from the evaluated fitness vector. This is the binding
  constraint for three straight cycles, so hit-rate cohorts only move
  the front via latency/metadata side effects.
- Item field-narrowing/removal does not move the measured metadata
  number; that figure is sourced from outside the overlaid struct.

## Open questions

- When does the scan/loop trace finally enter the evaluated vector? Until
  it does, no hit-rate change can be attributed (now a prerequisite, not
  optional).
- Where does the metadata-size number actually come from, given editing
  Item field widths does not change it?
- Is the cheap reuse-gate promotion actually being realized, or should it
  be dropped after two non-appearances?
- Can the O(1) tail-push latency win be decoupled from policy geometry,
  and the victim-selection lever's O(n) scan be replaced with a cheaper
  pick?
- Is the frequency-evict hybrid worth keeping while its hit-rate payoff
  is unmeasurable and it costs latency now?

## Per-policy notes

- Recency / LRU-style promotion: backfires on the scan/loop (LIRS-style)
  trace, but its promotion path is where the latency wins originated
  (O(1) tail insert). Keep the structural speedup, drop the recency
  retention bias here.
- Frequency / LFU-style promotion: cheap and metadata-neutral via the
  existing counter, but the strict pre-hit-max / ties-never rule
  under-fills the hot tier on loop traces where counts cluster, and the
  per-hit O(n) scan taxes latency.
- Reuse-distance / frequency-evict hybrid: current front-runner keeps
  recency promotion and evicts the least-reused resident; metadata-flat
  but latency-taxed, with its hit-rate benefit still unmeasurable.
