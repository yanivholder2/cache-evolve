## Recurring wins

- Trimming dead metadata off the eval path is cheap, monotonic, and
  repeatable: fields carried only by off-path policies peel off with zero
  hit-rate change and reclaim the 16-byte `Item` floor. Kept paying off
  cycle after cycle.
- Keeping state off the measured `Item` (in the unmeasured bucket struct)
  makes scan-resistance mechanisms "free" on the size axis — the size
  metric weighs only `Item`, so hit-rate and metadata-size stopped being
  antagonistic once state moved into the bucket.
- The index-preserving swap consistently collapses the quadratic p99 tails
  that key-matched reuse-bit rebuilds create, recovering clean tails with
  hit-rate held bit-for-bit identical. Proven across the cycles that used it.
- Genuine segment separation (2Q/SLRU-style) is the one lever that broke
  the front-admission hit-rate ceiling, and it does so at the metadata
  floor — trading only latency, not bytes.

## Recurring failures

- Frequency/LFU-based victim choice loses to recency-front every time it is
  tried: least-frequently-reused is a worse victim signal than plain
  LRU-front on this trace.
- Weakening permanent protection fails repeatedly — second-chance/CLOCK
  aging sheds immunity too fast and collapses back toward plain LRU;
  permanent binary protection fits this trace.
- Re-timing promotion on a single short bucket (single-step promotion,
  reuse-bit gating) buys latency at best and regresses hit-rate at worst —
  small buckets lack the slots for gradual promotion. Segmentation, not
  timing, is the lever.
- Admission geometry has a sharp optimum at the front slot; midpoint/wider
  probation is strictly worse.
- Overlay-hygiene debt recurs: single-file variants silently forfeit
  parent-layout wins (the 16-byte `Item`), producing metadata regressions
  by omission rather than logic.

## Open questions

- Can the segmented incumbent shed its per-access rotate/rebalance latency
  (e.g. lazy demotion) without the saturated-bucket hit-rate leak, since
  the rotation quietly maintains intra-protected eviction order?
- Where does the optimal protected-segment cap sit for these short buckets
  — near `max_size` with near-total-only demotion — and does that turn the
  aggressive-cap regression into a gain?
- Is two-segment separation the structural hit-rate limit for this trace,
  or is there headroom above the SLRU ceiling?
- How can overlay inheritance be enforced so prior-cycle layout wins are
  never dropped by variants that touch only one file?

## Per-policy notes

- LRU / front-admission: the settled base. Move-to-back-on-hit makes the
  frontmost slot always probationary, so explicit reuse protection and
  stateless front-admission are redundant re-encodings of one lever, not
  compounding ones.
- Reuse-bit / protection policies: permanent binary protection is the fit;
  keep state off `Item` and move it with index-preserving swaps. Protection
  adds value only over a policy that does not already positionally
  segregate reused items.
- SLRU / segmented: the current incumbent — only path to break the ceiling,
  delivered at the 16-byte floor. Frontier is now a hit-rate/latency trade;
  protected cap is real but sharply mis-tuned when aggressive.
- Frequency/LFU: dropped — worse victim signal than recency on this trace.
