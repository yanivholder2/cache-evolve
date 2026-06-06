# Cross-Cycle Insights

## Recurring wins

- Per-op latency wins come from removing hidden hot-path cost, not algorithmic
  cleverness: O(n) front-shifts, candidate-scan branches, `Instant::now()`
  syscalls, vtable dispatch, and an oversized keyed hasher each paid off when cut.
- Placement and footprint are orthogonal: MRU promotion and a slim `Item`
  stacked cleanly, banking faster ops and smaller entries without trading hit-rate.
- The structure-vs-layout pivot (flagged cycle 0002, cashed 0004+) is where real
  wins live once per-entry tuning saturates: relocating heat into a segment-side
  map reached the long-sought Pareto corner, and devirtualizing the inner policy
  gave the first harness-resolvable latency win.
- Trivializing the side-table hasher for integer keys was a locality-preserving
  latency win inside existing structure — proof that a "converged" frontier can
  still yield, when the cut removes work without fattening layout.
- Capacity pre-reservation and the direct-mapped/flat array are safe,
  hit-rate-neutral tail-variance reducers.

## Recurring failures

- Promoting more keys without a gate floods the probing tier and evicts resident
  keys; the hottest-only filter is load-bearing. Gate-shape and placement are
  substitutes, not additive — both target the same near-MRU warm mass.
- Chasing hit-rate on the hot `get` path (counter read-modify-writes, wider
  promotion cascades) costs real latency for noise-level gains.
- Micro-latency reorders (single scan per hit) sit below the measurement noise
  floor and move the wrong way as often as the right way.
- Stacking the banked size and latency winners on one hot path is substitutive:
  the side-table hash probe, not the dispatch, bounds get latency.
- Removing indirection that was buying locality backfires — outer-layer
  monomorphization fattened the `Vec` stride; a position-aligned parallel array
  paid an O(n) shift on every reorder. Eliminating the hash entirely lost to
  keeping a cheap hash.

## Open questions

- Is any get-path median lever left, or is get latency fully converged at the
  hasher floor with no probe-side headroom?
- The put-p99 tail resisted every lever and was pinned to eviction/cascade work,
  not the side-table touch — is that cascade reachable without disturbing gate
  decisions or hit-rate?
- Where does the next size win come from now that per-entry metadata is at its
  byte floor — further container/segment structure?

## Per-policy notes

- `ClassicLRU` is the only exercised eval policy and is purely positional: it
  never reads recency/frequency metadata, so `timestamp` and the LFU counter
  were pure overhead until trimmed, and frequency/admission/ghost-list levers
  moved hit-rate not at all.
- Under `ClassicLRU` on this trace, hit-rate is firmly capacity/compulsory-bound,
  not conflict-bound: placement, gate shape, associativity, and admission are all
  exhausted and confirmed capped. Treat hit-rate as closed; spend slots on the
  latency frontier (now the put/eviction path) and size structure instead.
