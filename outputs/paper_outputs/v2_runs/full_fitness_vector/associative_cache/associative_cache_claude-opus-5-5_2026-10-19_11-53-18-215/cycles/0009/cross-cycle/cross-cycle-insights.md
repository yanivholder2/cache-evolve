# Cross-Cycle Insights (cycles 0001–0009)

## Recurring wins

- **Single hashed bucket with one shared policy across the whole bucket.** Every key stays in its one hashed bucket, which keeps P2 slot_legality satisfied. A single policy over the full merged bucket beat split home/partner halves on hit rate (0001→0002) and is the base for every winner since.
- **SLRU with a large protected segment works well on this trace.** SLRU's hit-rate gains came without adding per-item metadata. Raising the protected share from 3/4 to 7/8 to 15/16 gained hit rate each time (0005–0009), but each step gained less, and the larger steps now cost tail latency.
- **Removing work beats adding cleverness.** Wins that held up came from deleting things: one hash per op, returning the insert index instead of re-finding, evicting only from probation with short rotates, putting cold keys at probation LRU (which skips a rotate), and dropping fields nobody reads.
- **Shrinking `Item` paid off on every path.** It went from 40 to 24 to 16 bytes (`{ key, value }`), because timestamp and LFU fields are never read under position-ordered policies. Each shrink made latency better, mostly at the tails, with hit rate unchanged.
- **Tag-first single-hash lookup.** A parallel 16-bit tag array, scanned forward before any full-key compare, lowered both get and put latency. Put gained because the ghost and eviction paths reuse the stored tags. Keeping the tags in sync cost less than the hashes and compares it removed (0005, 0007).
- **Ghost history helps when it only decides where a new key goes in probation.** Mid-probation entry, and later LRU entry for keys with no ghost hit, gained hit rate without displacing protected items (0004, 0008).
- **Lookup and replacement are separate axes.** Changes to one carried over cleanly onto the other. Merging single-axis winners from the previous cycle has been a reliable way to build the next base.

## Recurring failures

- **Placing a key outside its hashed bucket** (two-choice, cuckoo, cross-bucket victim areas) always fails the P2 slot_legality invariant.
- **Anything that shrinks or displaces protected.** This includes promoting ghost hits straight into protected (0003), ARC-style adaptive protected caps (0004), and approximate or bounded promotion inside protected (0007). Each one weakens scan resistance or demotes warm keys, and hit rate drops.
- **Making the ghost ring longer** (0006). Half a bucket of history is already enough. A longer ring adds false positives and makes the linear scan on the miss path longer.
- **Undersized frequency sketches for admission** (0009). The sketch saturates, almost every key counts as "seen", and admission falls back to MRU entry. That cancels the one-hit-wonder protection and makes put heavier.
- **Trying to make memmove cheaper.** This covered lazy promotion near MRU (0006), bounded protected rotates (0007), in-place rotates instead of shifts on the miss path (0009), and hot/cold layouts with indirection (0008). After the 16-byte stride, the bytes moved stopped mattering. These changes added branches and per-op work that cost as much as they saved, or more.
- **Wall-clock reads on the hot path.** `Instant::now()` hurt put and the tails.
- **Losing carry-forward.** Improvements were silently dropped when a variant branched from the wrong parent or missed a file (0005 lost the shrunken `item.rs`; 0006 lost tag-first find). Carry merged files over explicitly.
- **Unexplained latency changes.** Several cohorts saw get or put latency move with no code cause. Small per-op deltas need re-measuring before they get credit.

## Open questions

- **Admission with a signal that actually separates keys.** Candidates are a correctly sized and aged sketch, or ghost-hit counts / 2-bit ghost entries. Check that the signal discriminates before measuring hit rate.
- **Fixed per-op instructions on the get-hit path.** Latency now tracks branches and counter updates, not bytes moved. What else can be cut there?
- **Is 15/16 at 32+ worth its tail cost compared with 7/8?** Given LRU cold entry, would a wider probation do as well on hit rate with cheaper puts?
- **Plain LRU vs SLRU on the same base.** This has never been measured side by side; it has been carried forward unaddressed since 0002.
- **A non-linear ghost lookup** (hashed slot or bitset). Would it cut miss-path put cost at the current ring size without adding branches?
- **How sensitive is the cap/probation interaction to bucket size?** Small buckets (4–16 slots) behave differently from large ones.

## Per-policy notes

- **Split-half LRU (home/partner):** Hit rate was below baseline. It is superseded and should not come back.
- **Plain LRU over merged buckets:** It beat the split halves on hit rate. It has never been tested head to head against SLRU.
- **SLRU (probation/protected, boundary stored in `len0`):** This is the current lineage. Hit rate goes up with protected share, but the gains are saturating. Eviction should touch only probation. A thin probation (1 slot) means more evictions, ghost churn, and tail cost. The cap and probation floor are treated as settled as of 0009.
- **Ghost-guided placement (per-bucket fingerprint ring, about half a bucket long):** It works only for deciding where a key enters probation: ghost hit → MRU, otherwise LRU (earlier, the middle). Its costs are the linear scan on misses and writes on evictions. It needs at least 2 probation slots to make a difference.
- **ARC-style adaptive split:** It drifts toward plain LRU on this trace because the shrink signal fires often and the grow signal rounds to zero. It loses hit rate.
- **Frequency/LFU (count-min admission, `lfu_counter`):** The per-item LFU counter is dead weight and has been removed. Per-bucket count-min admission failed when undersized. It is still open if the sketch is sized and aged properly.
