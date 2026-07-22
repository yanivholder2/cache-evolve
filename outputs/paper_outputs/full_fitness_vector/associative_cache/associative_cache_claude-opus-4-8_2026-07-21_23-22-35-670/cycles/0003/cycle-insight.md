# Cycle 0003 Insight

## Cohort outcome

The cohort chased one prize from two directions — recover scan-resistant
hit-rate while holding the 16-byte metadata floor — plus a frequency-based
victim probe. Both floor-preserving paths landed as Winners: relocating the
reuse bit into the unmeasured bucket struct (v001) and buying scan
resistance from insertion geometry alone (v003). The LFU-tiebreak probe
(v002) was the sole loser.

## Patterns

- Scan resistance at the metadata floor is reachable two independent ways:
  explicit reuse state parked off the measured `Item` (v001) and pure
  probationary-front insertion with no per-item state (v003). Both hit
  hit-rate targets; the stateless geometric path was the clean one, while
  the stateful path recovered the same hit-rate but carried an
  implementation tax.
- The size metric weighs only `Item`, so state kept in the bucket is free
  on that axis — the exploit holds, and hit-rate and metadata-size stopped
  being antagonistic for these two Winners.
- The tax on v001 was implementation, not mechanism: a key-matched rebuild
  of the reuse bits on every hit is quadratic and blew out the tails, a cost
  an index-preserving swap would avoid.
- Frequency lost to recency again: evicting the least-frequently-reused item
  on saturated buckets was a worse victim signal than plain LRU-front,
  reaffirming that permanent binary protection fits this trace.
- Overlay hygiene bit v002: touching only the eviction file silently
  forfeited the parent's slim layout and regressed metadata — a win can be
  lost by omission, not just by logic.

## Open questions

- Can the bucket-bit path (v001) shed its quadratic reuse-bit rebuild via
  index-preserving swaps and then match or beat the stateless geometric
  path's clean tails?
- Does stateless probation (v003) leave any hit-rate on the table versus
  explicit reuse tracking, or is insertion geometry alone sufficient here?
- How to keep single-file variants from silently dropping prior-cycle wins
  carried in files they don't touch?

## Continuation

Treat v003's stateless probationary-front insertion as the cheap incumbent
to build on — it lifts hit-rate at the floor with better tails and no per-item
state. If pursuing v001's bucket-bit path, first replace the key-matched
rebuild with an index-preserving swap so the bits move for free. Drop
frequency/LFU-based victim choice; recency-front wins. Guard variants against
inheriting only part of the parent's layout.
