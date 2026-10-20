## Recurring wins

- Latency wins come from removing hidden hot-path cost, not cleverness:
  eliminating an `Instant::now()` syscall, an O(n) front-shift, a
  candidate-scan branch, vtable dispatch on the inner policy, and — biggest —
  the keyed SipHash permutation on integer keys (swapped for a trivial `u64`
  hasher) all paid off cleanly.
- Structural change beats per-entry tuning *when it preserves locality*:
  devirtualizing the inner policy and trivializing the side-table hasher both
  cleared the noise floor; relocating the heat signal out of `Item` into a
  segment-side map reached the Pareto corner (best hit-rate AT the metadata
  floor).
- Placement (MRU / `InsertPosition::Back`) was the early, durable hit-rate win
  and captures most protectable warm mass on this trace.
- Capacity pre-reservation / ~2× sizing is a safe, hit-rate-neutral tail/variance
  reducer (cuts rehash spikes and slot-collision perturbation).
- Metadata slimming to the byte floor was free under the active policy and
  composed orthogonally with placement.

## Recurring failures

- Every hit-rate lever beyond placement keeps returning null: wider/reshaped
  promotion gates, frequency gating, raised associativity, and admission/ghost
  lists all left hit-rate at the recency-only ceiling — it is
  capacity/compulsory-bound, not conflict-bound.
- Micro-latency moves sit below the harness noise floor and often regress:
  branch reorders and deferring "wasted" index arithmetic moved metrics the
  wrong way.
- Stacking the two banked Pareto winners (side-table size + devirtualization)
  is substitutive, not additive — they compete on the shared get path.
- Removing indirection by *fattening layout* backfires: outer-Bucket
  monomorphization and a position-aligned parallel `Vec<u8>` both regressed by
  hurting locality / adding O(n) reorder shifts.

## Open questions

- Is there a conflict-resolution path that does NOT terminate in the stash, so
  the put-tail win and the get-median cost decouple instead of trading?
- Residency cleared the long-standing cap only by a hair at steep put-p99 cost —
  is any meaningful hit-rate above it reachable at tolerable tail, or is the cap
  the practical ceiling?
- Are the characterized trades (shallow cascade: put-p99↓ / hit-rate↓; deeper
  cascade: hit-rate↑ / put-p99↑; relocated put clear: p50↓ / tails↑) worth
  banking, or only worth a strictly-free version?

## Per-policy notes

- ClassicLRU (active eval policy) is purely positional/recency: it never reads
  `timestamp` or the LFU counter, which made those fields pure size+latency dead
  weight and enabled metadata slimming. Hit-rate is effectively capped under it
  across placement, gate, associativity, and admission axes.
- Dash/side-table segment: get median is hasher/probe-bound (a hard floor once
  the hasher is trivialized), and the median get hit actually resolves in the
  STASH, not the promoted tiers. The stash is a shared resource coupling
  put-side cascade work to get-side latency — cascade depth is a bidirectional
  latency↔hit-rate dial, and any cascade economy ending in the stash pays at the
  get median. The put-p99 tail is owned by eviction/cascade work, not the
  side-table touch.
