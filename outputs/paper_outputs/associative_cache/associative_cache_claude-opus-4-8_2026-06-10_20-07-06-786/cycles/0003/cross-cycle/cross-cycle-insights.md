## Recurring wins

- Victim/admission logic inside the bucket is the high-leverage surface every
  cycle so far — wins came from changing *which* entry is evicted or *when* a
  key earns protection, never from geometry or metadata plumbing.
- Demanding proof of reuse before granting protection keeps paying off:
  frequency-first eviction (C1), promote-on-second-hit segmentation (C2), and
  giving rising keys a full lap to earn promotion via FIFO probation (C3) all
  won by shielding the reusable working set from one-hit traffic.
- Repurposing the dormant `lfu_counter` as a live signal (hit count, then
  promote flag) has been the substrate for multiple winners.
- Probation eviction *order* is itself a lever: reversing newest-first to
  oldest-first (FIFO) recovered keys that were being discarded mid-rise.

## Recurring failures

- Stacking the orthogonal levers does not compose — front admission plus
  frequency-first eviction conflict, making a fresh key simultaneously oldest
  and minimum-frequency, so it is evicted before it can rise.
- Counter aging / staleness handling regresses: the trace is collision-bound,
  and "once-hot immortality" is a feature here, so decaying durable counts
  removes exactly the protection that makes high-frequency keys valuable.
- Capacity caps with demotion (protected-segment cap) are inert dead code on
  this trace — the protected segment never saturates — and only add scan cost.
- Geometry coarsening (wider buckets) gave the smallest gain and is
  collision-bound; returns are already diminishing.

## Open questions

- Per-op cost remains structurally present but largely unmeasured: the
  O(bucket_size) linear victim/probation scan is paid by winners and losers
  alike. Can it be made O(1) (tracked segment boundary) without losing the
  hit-rate gain?
- Can cost-saving inert changes (reorder-free tiebreak) be banked *alongside*
  the FIFO-probation hit-rate win, rather than chosen against it?
- Is there a workload regime (shifting working set) where staleness would
  finally dominate, or is this trace permanently collision-bound?
- With probation order and admission tuned, is segment sizing the last
  remaining hit-rate axis, or is the scheme near its ceiling here?

## Per-policy notes

- LRU (pure recency): consistently beaten — recency alone admits scan/one-hit
  traffic the working set needs shielded from.
- LFU / frequency-first victim selection: the first breakout win; durable
  counts protect hot keys, but the approach is brittle when composed with
  admission changes or weakened by aging.
- Segmented-LRU (promote-on-second-hit): the current baseline lineage and
  strongest scan-resistant family; gains now come from internal details
  (probation order = FIFO) rather than the segmentation idea itself.
- Across all policies the binding constraint is capacity collision, not
  staleness; fitness vectors still report hit-rate only, leaving the cost
  side of every trade untracked.
