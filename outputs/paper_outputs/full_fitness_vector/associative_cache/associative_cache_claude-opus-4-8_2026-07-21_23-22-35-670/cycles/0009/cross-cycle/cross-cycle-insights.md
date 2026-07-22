# Cross-Cycle Insights

## Recurring wins

- Trimming dead metadata off the eval path is monotonic and cheap: fields
  carried only by off-eval-path policies peel away with zero hit-rate cost,
  repeatedly reclaiming the metadata floor.
- Keeping state off the measured `Item` (in the bucket struct, a parallel
  vector, or a scalar mask) makes scan-resistance free on the size axis —
  hit-rate and metadata stopped being antagonistic once this exploit was found.
- Index-preserving relocation of reuse state — mirroring the policy's own
  single-element move instead of rebuilding by key-search — reliably collapses
  the p99 tails while holding hit-rate bit-for-bit. It paid off every cycle it
  was applied.
- Genuine two-segment separation (SLRU-style protected segment) is the one
  structural change that broke the front-admission hit-rate ceiling, and it
  did so at the metadata floor.
- Compressing the reuse set into a scalar `u64` mask under small associativity
  turns per-hit heap shifts and eviction scans into branch-free register ops —
  the latest capturable latency win.

## Recurring failures

- Plain associativity/bucket-width increases: conflict-miss headroom is thin
  and wider buckets coarsen the hash spread, so hit-rate falls while the linear
  scan cost rises — set diversity beats per-set depth here.
- Frequency/LFU-based victim choice and ghost/2Q cross-eviction memory: both
  lose to plain recency-front, over-inflating protection and eroding the reuse
  discrimination that earned the ceiling.
- Weakening permanent binary protection (second-chance/CLOCK aging, in-residency
  two-hit thresholds, aggressive protected caps): every variant sheds immunity
  too fast or delays protecting genuinely hot keys, and regresses.
- Optimizing sub-resolution fixed per-op costs (hash-function swaps, single-element
  rotations below the ~100ns timer): correct but unrewardable — only heap-vector
  paths are harness-resolvable.
- Overlay hygiene by omission: single-file variants silently revert a prior
  cycle's slim `Item` layout, losing a win through inheritance rather than logic.

## Open questions

- Is the two-segment SLRU hit-rate now the structural ceiling for this trace, or
  does headroom exist above it? Three independent headroom bets have all regressed.
- Where does the protected cap actually optimize? Both extremes are falsified and
  the mid-range reads nearly flat — is the default already optimal?
- With the reuse vector gone scalar, are the remaining tails purely the `items`
  front-shifts (`insert(0)`/`remove(0)`), and is any harness-resolvable latency
  gain left there?
- Can the per-access protected rotation be shed for latency without the
  saturated-bucket eviction-order leak (e.g. lazy demotion only when fully protected)?

## Per-policy notes

- LRU / front-admission: stateless probationary-front insertion is the cheap
  incumbent; the front slot is the settled admission-geometry optimum (midpoint
  and wider-probation variants regress).
- SLRU / segmented: the ceiling-breaking incumbent, delivering best hit-rate at
  the 16-byte floor. Gains cost latency (per-access rotate/rebalance), not
  metadata; protected cap is a flat lever exhausted at both extremes and mid-range.
- Reuse-protection: permanent one-hit binary protection is the right fit; it is
  redundant with front-admission under move-to-back-on-hit, so it only adds value
  over a policy that does not positionally segregate reused items.
- LFU / frequency & 2Q ghost memory: consistently worse victim signals than
  recency-front on this trace; do not revisit.
