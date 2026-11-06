# Cycle 0005 Insight

## Cohort outcome

Two speed variants started from cycle 0004's 24-byte `Item`. v001 replaced the global atomic tick with a per-bucket counter. v002 added a dense 1-byte fingerprint pre-filter to the get scan. A third, policy variant (v003) made LIP insertion depth adapt per bucket. All three are Winners. v001 and v002 made clear per-op gains with hit rate exactly unchanged. v003 is the weakest: its hit-rate summary fell, and it only had small gains in a couple of cells.

## Patterns

- Both long-open speed questions are now answered, and both answers are positive. The cycle 0004 regression came from the shared atomic clock, not from the environment or the denser layout. With the atomic gone, the 24-byte `Item` is at least as fast as the old baseline. The fingerprint pre-filter, recommended for three cycles, is the first change that actually reduced full-key compares. It sped up get and put at both p50 and p99.
- Density is what made the side array work. A `u64` key mirror failed in cycle 0002, while a byte-tag stream 24× denser than items, scanned in vectorisable chunks, succeeded. Misses dominate, and every new-key put runs a miss scan, so the miss path is where the savings land. A small byte memmove on hits is an acceptable cost.
- The two speed wins are independent and have not been combined. v002 still carries the atomic tick that v001 removed. Their gains come from separate costs (the shared-line RMW on put, and compare count on scan), so stacking them should give both.
- Hit position inside one LRU-ordered bucket is not a usable reuse signal. Hits below the insertion point mix new admits with old items that have aged down. As a result, adaptive depth read "pressure" and admitted keys deeper, which hurt the hm_0 cell it was meant for and also web_0 scan resistance. This is the third reuse or adaptation signal in a row to fail the two-workload tradeoff. The fixed quarter-depth LIP with BIP 1/32 is still the best policy.
- Metadata-size is still blind to per-bucket state. Tags in v002 and extra bucket fields in v003 both added real footprint, and the dimension reads flat for both.

## Open questions

- Do the per-bucket clock and the fingerprint filter stack additively? Can the timestamp stores be skipped completely when no active policy reads them?
- Can tags be extended beyond the ClassicLRU/LIP path, for example to CLOCK? Would wider tags, or SIMD on the 16-wide mask, cut the remaining false-positive compares?
- Is fused rotation with `ptr::copy` (cycle 0004 v001, never tested cleanly) worth retrying now that the atomic is gone, so the put tail can be measured on its own?

## Continuation

Merge v001 and v002 into one base: the 24-byte `Item`, a per-bucket non-atomic clock (or no stamp at all when no policy reads it), and the byte-tag pre-filter. Then measure the cleaned-up fused rotation on top of that base to attack the remaining put cost from the memmove. Drop adaptive depth based on hit position. Do not attempt more reuse or adaptation signals unless one carries an explicit per-item probationary bit with symmetric grow and decay. Otherwise, keep the policy frozen and spend the next cycles on speed.
