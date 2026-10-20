## Recurring wins

- **One hashed bucket per key, with one shared policy over a merged bucket.** This has been the base since cycle 0002. Designs that place keys in several buckets cannot pass P2 slot_legality. Split per-half policies held hit rate down until they were merged.
- **Cutting data movement is the main latency lever.** Cost is mostly memmove of items inside the bucket. Several fixes made get and put faster without changing hit rate, and they stack:
  - narrower `Item` strides: 40 → 24 → 16 bytes `{ key, value }`
  - evicting only from probation, with short rotates
  - returning the insert index instead of re-finding it
  - The biggest gains show up in the tails, where the longest shifts happen.
- **Removing unused metadata is free.** Under position-ordered policies, timestamp and LFU fields are never read. Removing them brought metadata-size to its floor with no hit-rate cost. Taking wall-clock reads off the hot path helped put latency and tails.
- **SLRU scan resistance pays on this trace, and more protected capacity raises hit rate.** The cap went 3/4 → 7/8. From cycle 0006 on, this costs get and put latency (see Per-policy notes).
- **Ghost history helps only when it decides where a key enters probation.** Cold keys go mid-probation and ghost hits go to MRU. Used this way it gained hit rate without displacing protected items.
- **Changing one axis per variant from a shared parent works well.** Each Winner isolates one lever: get-median, hit-rate, or metadata/tails. That makes merging predictable.

## Recurring failures

- **Cross-bucket placement** (two-choice, cuckoo, victim areas) fails P2 slot_legality every time. Do not propose it.
- **Anything that shrinks or displaces protected loses hit rate:**
  - ghost-to-protected admission
  - ARC-style adaptive protected caps, which drift toward LRU because the shrink signal dominates and grow steps round to zero in small buckets
  - split home/partner halves
- **Skipping shallow moves**, such as lazy promotion near MRU, saves nothing, because hot keys already sit near MRU. The real cost is in deep protected hits and probation promotions.
- **Scan direction that does not match the segment layout.** MRU-first scans make every probation hit walk all of protected first, which hurt the get median. Credit that was once given to scan order belonged to the shorter moves.
- **Enlarging the ghost ring.** Half a bucket already captures the usable reuse. A longer ring adds false positives and makes the linear O(ring) scan longer on every miss-path put.
- **Fragile carry-forward.** Improvements were lost or not merged several times: the 24-byte item, the 16-byte item, and the tag-first `find` from cycle 0005, which is still unmerged. Some regressions came from a missing carry-forward, not from the idea being tested.
- **Latency shifts with no code cause** show up almost every cycle. Small per-op deltas are unreliable until re-measured side by side with the parent.

## Open questions

- Can ring offsets or swap-based promotion make deep protected and probation moves cheap enough to keep 7/8's hit rate without its latency cost?
- On the 16-byte item with a 7/8 cap, how much does keeping the tag array in sync cost on put, compared with the get-median gain and the removed second hash?
- Can a non-linear ghost lookup (a bitset or a hashed slot) with a fingerprint taken from the existing hash shrink the put tail on the miss path?
- Where is the best point on the hit-rate/latency curve for the protected cap: between 3/4 and 7/8, or above 7/8 on large buckets? How sensitive is the probation floor to bucket size?
- Plain LRU vs SLRU has never been measured head to head on the same base. It has been carried forward unaddressed since cycle 0002.
- Is mid-probation the best entry point for cold keys?

## Per-policy notes

- **Split-half LRU (home/partner):** worse hit rate than one shared policy, and the partner path adds tail latency. Retired.
- **Plain LRU over a merged bucket:** recovered hit rate above the original baseline in cycle 0002. It has not been tested against SLRU since, and is likely dominated by SLRU on this LIRS-style trace.
- **SLRU (probation/protected, boundary stored in `len0`):** the current base.
  - Hit rate rises with protected share.
  - Latency is set by how long rotates are. Larger protected means longer protected-hit rotates. A 1-slot probation means more frequent whole-bucket shifts on eviction and ghost re-insert.
  - Evict from probation only, and keep moves out of protected wherever possible.
- **SLRU with ghost ring:** helps only through probation placement.
  - Using it for protected admission or adaptive cap sizing hurts hit rate.
  - Its put-tail cost (second hash, linear ring scan, ring writes) is the main overhead left to reduce.
  - Its per-bucket memory does not show up in the metadata-size dimension.
- **ARC-style adaptive split:** lost hit rate. Retired.
