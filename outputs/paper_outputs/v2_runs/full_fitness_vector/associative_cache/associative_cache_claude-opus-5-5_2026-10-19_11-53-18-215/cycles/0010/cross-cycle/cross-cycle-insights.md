# Cross-Cycle Insights (cycles 0001–0010)

## Recurring wins

- **Work inside one bucket with one shared policy.** Merged buckets where every key can use the whole bucket under a single policy (plain LRU at first, SLRU after that) brought back the hit rate that split-half designs had lost (0001→0002). Every later Winner builds on this base.
- **SLRU with a large protected segment drives hit rate.** Raising protected from 3/4 to 7/8 to 15/16 gained hit rate each time (0005–0009), with smaller gains at each step. The lever is now saturated.
- **Changes that remove work pay off reliably.** Examples: hashing once, the returned insert index, evicting only from probation with short rotates, dropping dead `Item` fields to reach a 16-byte `{key, value}`, cold keys entering at probation LRU (which drops a rotate), and the branch-free hit path. These changes stack and almost never cost hit rate.
- **Tag-first lookup.** A parallel 16-bit tag array, scanned before any full-key compare, lowered the get median. Reusing stored tags on the ghost and eviction paths also made put faster. Keeping tags in sync is cheap (0005, 0007).
- **Lookup and replacement are orthogonal.** Layout, lookup and metadata changes carry over cleanly onto policy changes, and the reverse holds too. Merges that keep the replacement decisions bit-identical reproduce the expected hit rate.
- **Ghost history helps only as a placement hint.** Using a ghost hit to choose where in probation a key enters gained hit rate (0004). LRU entry for cold keys with no ghost hit gained more (0008).
- **Shortening the longest hit-path move cuts the tail.** Probation-only eviction (0003) and midpoint promotion (0010) gave the largest put/get tail gains.

## Recurring failures

- **Placing keys outside their hashed bucket** (two-choice, cuckoo, cross-bucket victim areas). These designs cannot pass the P2 slot_legality check.
- **Anything that shrinks or displaces protected**: ghost-to-protected admission (0003), ARC-style adaptive caps (0004), and approximate or bounded promotion that demotes warm keys (0007). All of them lose scan resistance on this LIRS-style trace.
- **Undiscriminating admission signals.** A count-min sketch that was too small and aged too slowly saturated, fell back to MRU entry and lost hit rate (0009). A signal has to be shown to tell keys apart before it is measured.
- **Ghost-structure tuning**: a larger ring (0006) and a direct-mapped O(1) table (0010). Neither saved latency and both cost hit rate. How precisely ghosts age matters more than how fast they are looked up.
- **Skipping cheap work**: lazy promotion near MRU (0006), MRU early return, branch-free ghost scans and removing the miss-path memmove (0009). The skipped work was already near-free, and the extra branches cost more than it saved.
- **Layout indirection**: a hot/cold bucket split and boxed cold state (0008). Saving memory touches does not pay once the working set fits in cache.
- **Split-half policies inside a bucket** (0001): the bookkeeping cancelled out any gain in flexibility.
- **Fixes the code can't explain.** Latency changed in several cycles with no code cause. Small per-op changes need a repeat measurement before they get credit.

## Open questions

- Does merging cycle 0010's branch-free shared rotate with midpoint promotion keep both tail gains, or does the per-kind rotate end bring a branch back?
- The get median now looks set by `find_tagged` plus the base rotate. Can a SIMD or narrower tag compare lower it?
- Hit-rate headroom: the cap, the probation floor and ordering inside protected are all used up. The remaining untested option is a discriminating admission signal, such as ghost-hit counts, 2-bit ghost entries, or a sketch that is correctly sized and aged and checked for saturation first.
- Is the small hit-rate gain from 15/16 at large buckets worth its tail cost now that midpoint promotion has won back much of that tail?
- Carry-forward is fragile: Winners have repeatedly failed to reach the next cohort (tags in 0006, `item.rs` in 0005). Merges should be explicit and checked.

## Per-policy notes

- **Split-half home/partner LRU:** dead end. It held hit rate below baseline.
- **Plain LRU over merged buckets:** beat split halves, but SLRU replaced it. It was never measured directly against SLRU on the same base.
- **SLRU (current lineage):**
  - Segment membership is stored in the `len0` boundary, so it needs no per-item metadata. Timestamp and LFU fields are dead weight.
  - Hit rate rises with protected share up to about 15/16 and then flattens. Probation must keep at least 2 slots for the entry-position lever to matter.
  - Cold keys enter best at probation LRU. Ghost hits enter at MRU or mid-probation.
  - Midpoint promotion into protected is neutral for hit rate and good for the tail. Reordering protected any further is exhausted.
  - Per-op cost is now instructions and branches. Bytes moved matter only for long hit-path rotates; miss-path memmove is not a bottleneck.
- **ARC-style adaptive split:** failed. The shrink signal fires often while the grow signal rounds to zero in small buckets, so the policy drifts toward LRU.
- **Frequency-gated (count-min) admission:** failed as built because the sketch saturated. Not yet ruled out if it is correctly sized and aged.
