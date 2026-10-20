## Recurring wins

- **Merged single bucket with one shared policy (cycles 1–4).** A key lives only in its one hashed bucket, and the whole bucket uses one policy. This is the only shape that passes P2 slot_legality, and it has been the base of every Winner since cycle 1.
- **Cutting data movement makes get and put faster without changing hit rate (cycles 1–4).** The changes that helped: hash once per op, contiguous bucket storage, in-place policy updates, a narrower 24-byte `Item`, evicting only from probation with short rotates, and returning the insert index instead of re-finding. They stacked every time they were combined. Moves that cross protected are the expensive ones.
- **Taking dead or costly fields off the hot path is free (cycles 2–3).** Replacing the wall-clock timestamp with logical ticks made put faster. Under position-ordered policies, `timestamp`/`lfu_counter` are never read. Removing them shrank metadata at no hit-rate cost.
- **Scan resistance pays on this LIRS-like trace (cycles 2–4).** SLRU with a strong protected segment beat both split-half LRU and its own parent on hit rate.
- **Ghost history helps when it only decides placement inside probation (cycle 4).** Ghost hits enter at MRU and cold keys enter mid-probation. This gained hit rate and never displaces protected.

## Recurring failures

- **Putting keys outside their hashed bucket (cycle 1).** Two-choice, cuckoo and cross-bucket victim areas all fail P2 slot_legality every time. This direction is closed.
- **Splitting a bucket into home/partner halves (cycles 1–2).** The bookkeeping cancelled the extra flexibility, hurt hit rate and added a partner-path tail.
- **Anything that weakens or displaces protected (cycles 3–4).** Ghost-to-protected admission and an ARC-style adaptive protected cap both lost hit rate. The cap drifted toward plain LRU: ghost hits shrank it often, and its grow signal rounds to zero in small buckets.
- **Lookup order that ignores the segment layout (cycles 2, 4).** Full-bucket scans and MRU-end scans that walk all of protected before reaching probation keep pushing up the get median.
- **Giving credit to unexplained latency shifts (cycles 3–4).** Small changes in per-op time with no code cause showed up and later went away. Re-measure before keeping a change for latency alone.

## Open questions

- **Segment-aware or forward `find`.** Does either recover the get median while keeping the stacked put gains?
- **Cheaper ghost path.** Can the fingerprint come from the existing hash, with no second hash? Can the ring be checked without a linear scan? Can the ghost-hit and `len0 == 0` branches use the short-rotate eviction? Each would reduce ghost's put-tail cost.
- **Cold-entry position and ghost-ring size.** Is mid-probation the best entry point for cold keys? Would a larger ring catch longer reuse distances? Note that ring memory is not counted in the metadata-size dimension.
- **Plain LRU vs SLRU, and fixed protected fractions other than 3/4.** Still untested on a common base after four cycles.
- **Remaining protected-wide moves.** Can ring offsets or swap-based promotion remove the rotate across protected on a probation hit?

## Per-policy notes

- **Split-half LRU (home/partner):** Dropped. It held hit rate down and added a tail.
- **Plain LRU over merged bucket:** It lifted hit rate above baseline, but it has never been compared with SLRU directly. It also lacks scan resistance, which this trace rewards.
- **SLRU (probation + protected, 3/4 protected):** Current base policy. Segment membership lives in the `len0` boundary, so it adds no per-item metadata. Its cost comes from movement, not from the policy. Keep evictions and rotates inside probation.
- **SLRU + ghost placement:** Placement-only use is the current best hit-rate lever. It costs a second hash, a ring scan on every miss and a ring write on every eviction, all on the put tail.
- **SLRU + ghost → protected / adaptive cap (ARC-like):** Both lost hit rate. A ghost hit is weaker evidence than a resident second hit, so it should not shrink or invade protected.
