## Recurring wins

- **Single-bucket rule plus one shared policy over a merged bucket.** Since cycle 0002 every Winner has kept each key in its one hashed bucket and run a single policy across the whole bucket. Keys that can use the full bucket gain hit rate, and the partner-path tail goes away.
- **The main latency cost is data movement, so remove work.** Several cycles show the same thing: shorter moves are what pay off. Narrower `Item` strides (40 → 24 → 16 bytes), evicting only from probation, short rotates, returning the insert index instead of re-finding, and entering cold keys at LRU (which drops a rotate) all made things faster with hit rate unchanged or better. The biggest gains show up at the tails.
- **Metadata shrinks for free under position-ordered policies.** SLRU never reads the timestamp or LFU fields. Shrinking them over three cycles and then removing them (`{ key, value }`) brought metadata to its floor at no cost to hit rate. Wall-clock reads on the hot path were also pure cost.
- **Tag-first lookup with a single hash.** Scanning a parallel 16-bit tag array before comparing full keys improved the get median. Put also got faster, because the ghost and eviction paths reuse the stored tags. Keeping the tags in sync costs less than the hashes and compares it saves.
- **Lookup and replacement are orthogonal.** Changes on one axis held the other fixed and merged cleanly: tag-find with the cap, and stride with the policy. Merging Winners that each changed one axis has been reliable, as long as the files are carried forward explicitly.
- **Protected strength drives hit rate on this LIRS-like trace.** SLRU beat plain LRU over the merged bucket. Raising the protected share (3/4 → 7/8 → 15/16) added hit rate each time, but the gain got smaller at each step.
- **Ghost history works as a placement signal only.** Using ghost hits to choose where a new key enters probation gained hit rate. So did sending ghost-miss cold keys to probation LRU, which was the first hit-rate gain that did not touch the cap.

## Recurring failures

- **Placing a key outside its hashed bucket** (two-choice, cuckoo, victim areas) always fails the P2 slot_legality invariant.
- **Anything that shrinks or displaces protected loses hit rate and scan resistance.** Three attempts failed this way: ghost-to-protected admission, an ARC-style adaptive cap, and approximate or bounded promotion that demotes warm keys. A ghost hit is weaker evidence than a resident second hit.
- **Split-half (home/partner) LRU** inside a bucket added bookkeeping and lowered hit rate.
- **Skipping or bounding protected rotates** (lazy near-MRU promotion, move-ahead-k) saved nothing. Protected moves on 16-byte items are already cheap and hot keys sit near MRU anyway. Bounding them added variable-length moves and churn on the put tail.
- **A bigger ghost ring** added false positives and a longer linear scan on every miss. A ring of about half a bucket is enough.
- **Layout indirection** (hot/cold split, boxed cold state, inline/heap branches) moves put cost around without lowering get. Once tags are used, the get working set is cache-resident.
- **Scan order that ignores segment layout.** Scanning MRU-first made every probation hit walk all of protected first, which hurt the get median.
- **Unmerged or lost carry-forwards** have repeatedly hidden gains or created fake regressions. Examples: the tag-find sat unmerged for two cycles, and the `item.rs` shrink was dropped once.
- **Latency shifts with no cause in the code** have shown up in most cycles. Small per-op deltas are not reliable without a side-by-side re-run.

## Open questions

- **Cap versus admission.** Do the 15/16 cap and probation-LRU cold entry stack? At 16-slot buckets the 15/16 cap leaves only one probation slot, where LRU entry and mid entry are the same. Would a probation floor of 2, or a smaller cap combined with LRU entry, beat 15/16?
- **Stronger admission signals.** Would a frequency sketch or a ghost-hit count beat a single ghost bit, for example by keeping one-hit-wonders out of probation entirely?
- **Miss-path cost.** This has gone unmeasured across several cycles. How much does the linear ghost scan cost, and how much do whole-bucket insert/remove shifts cost? Would a hashed or bitset ghost lookup, or a ring-offset layout for probation only, remove them?
- **Per-hit instruction cost on get**, now that cache locality is no longer the limit.
- **Bucket-size sensitivity** of the cap and probation floor across 4, 8, 16 and larger slot counts.

## Per-policy notes

- **Plain LRU (merged bucket):** recovered hit rate over split-half and beat baseline, but SLRU did better. It was never re-measured against SLRU on the same base.
- **Split-half LRU:** the hit-rate floor of this lineage. Dropped.
- **SLRU (current lineage):** segment membership comes from a boundary index, so it needs no per-item metadata. It is scan-resistant, and a larger protected share helps with diminishing returns. Eviction only from probation, rotates kept short, protected moves cheap. The cap now sits at 15/16 on large buckets and 7/8 otherwise, and the probation width is the coupling point with admission.
- **SLRU + ghost ring:** helps only through probation placement (mid entry for ghost hits, LRU entry for cold keys). Harmful when it feeds protected admission or protected sizing. Its cost is the linear ring scan and ring writes on the miss and evict path. The ring size is saturated.
- **ARC-style adaptive split:** drifted toward LRU (frequent shrink signal, grow signal rounds to zero slots in small buckets) and lost hit rate. Dropped.
- **Multi-bucket placement (two-choice/cuckoo):** illegal under P2. Dropped.
