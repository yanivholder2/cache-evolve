# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-20T09:56:02+00:00

## timeline

### 09:56:25 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (cycles 0001–0009)\n\n## Recurring wins\n\n- **Single hashed bucket with one shared policy across the whole bucket.** Every key stays in its one hashed bucket, which keeps P2 slot_legality satisfied. A single policy over the full merged bucket beat split home/partner halves on hit rate (0001→0002) and is the base for every winner since.\n- **SLRU with a large protected segment works well on this trace.** SLRU's hit-rate gains came without adding per-item metadata. Raising the protected share from 3/4 to 7/8 to 15/16 gained hit rate each time (0005–0009), but each step gained less, and the larger steps now cost tail latency.\n- **Removing work beats adding cleverness.** Wins that held up came from deleting things: one hash per op, returning the insert index instead of re-finding, evicting only from probation with short rotates, putting cold keys at probation LRU (which skips a rotate), and dropping fields nobody reads.\n- **Shrinking `Item` paid off on every path.** It went from 40 to 24 to 16 bytes (`{ key, value }`), because timestamp and LFU fields are never read under position-ordered policies. Each shrink made latency better, mostly at the tails, with hit rate unchanged.\n- **Tag-first single-hash lookup.** A parallel 16-bit tag array, scanned forward before any full-key compare, lowered both get and put latency. Put gained because the ghost and eviction paths reuse the stored tags. Keeping the tags in sync cost less than the hashes and compares it removed (0005, 0007).\n- **Ghost history helps when it only decides where a new key goes in probation.** Mid-probation entry, and later LRU entry for keys with no ghost hit, gained hit rate without displacing protected items (0004, 0008).\n- **Lookup and replacement are separate axes.** Changes to one carried over cleanly onto the other. Merging single-axis winners from the previous cycle has been a reliable way to build the next base.\n\n## Recurring failures\n\n- **Placing a key outside its hashed bucket** (two-choice, cuckoo, cross-bucket victim areas) always fails the P2 slot_legality invariant.\n- **Anything that shrinks or displaces protected.** This includes promoting ghost hits straight into protected (0003), ARC-style adaptive protected caps (0004), and approximate or bounded promotion inside protected (0007). Each one weakens scan resistance or demotes warm keys, and hit rate drops.\n- **Making the ghost ring longer** (0006). Half a bucket of history is already enough. A longer ring adds false positives and makes the linear scan on the miss path longer.\n- **Undersized frequency sketches for admission** (0009). The sketch saturates, almost every key counts as \"seen\", and admission falls back to MRU entry. That cancels the one-hit-wonder protection and makes put heavier.\n- **Trying to make memmove cheaper.** This covered lazy promotion near MRU (0006), bounded protected rotates (0007), in-place rotates instead of shifts on the miss path (0009), and hot/cold layouts with indirection (0008). After the 16-byte stride, the bytes moved stopped mattering. These changes added branches and per-op work that cost as much as they saved, or more.\n- **Wall-clock reads on the hot path.** `Instant::now()` hurt put and the tails.\n- **Losing carry-forward.** Improvements were silently dropped when a variant branched from the wrong parent or missed a file (0005 lost the shrunken `item.rs`; 0006 lost tag-first find). Carry merged files over explicitly.\n- **Unexplained latency changes.** Several cohorts saw get or put latency move with no code cause. Small per-op deltas need re-measuring before they get credit.\n\n## Open questions\n\n- **Admission with a signal that actually separates keys.** Candidates are a correctly sized and aged sketch, or ghost-hit counts / 2-bit ghost entries. Check that the signal discriminates before measuring hit rate.\n- **Fixed per-op instructions on the get-hit path.** Latency now tracks branches and counter updates, not bytes moved. What else can be cut there?\n- **Is 15/16 at 32+ worth its tail cost compared with 7/8?** Given LRU cold entry, would a wider probation do as well on hit rate with cheaper puts?\n- **Plain LRU vs SLRU on the same base.** This has never been measured side by side; it has been carried forward unaddressed since 0002.\n- **A non-linear ghost lookup** (hashed slot or bitset). Would it cut miss-path put cost at the current ring size without adding branches?\n- **How sensitive is the cap/probation interaction to bucket size?** Small buckets (4–16 slots) behave differently from large ones.\n\n## Per-policy notes\n\n- **Split-half LRU (home/partner):** Hit rate was below baseline. It is superseded and should not come back.\n- **Plain LRU over merged buckets:** It beat the split halves on hit rate. It has never been tested head to head against SLRU.\n- **SLRU (probation/protected, boundary stored in `len0`):** This is the current lineage. Hit rate goes up with protected share, but the gains are saturating. Eviction should touch only probation. A thin probation (1 slot) means more evictions, ghost churn, and tail cost. The cap and probation floor are treated as settled as of 0009.\n- **Ghost-guided placement (per-bucket fingerprint ring, about half a bucket long):** It works only for deciding where a key enters probation: ghost hit → MRU, otherwise LRU (earlier, the middle). Its costs are the linear scan on misses and writes on evictions. It needs at least 2 probation slots to make a difference.\n- **ARC-style adaptive split:** It drifts toward plain LRU on this trace because the shrink signal fires often and the grow signal rounds to zero. It loses hit rate.\n- **Frequency/LFU (count-min admission, `lfu_counter`):** The per-item LFU counter is dead weight and has been removed. Per-bucket count-min admission failed when undersized. It is still open if the sketch is sized and aged properly.\n"
}
```

### 09:56:32 chat

I wrote `cross-cycle-insights.md` in the cycle 0009 cross-cycle directory. It has the four required sections in order and was built only from the ledger for cycles 0001–0009.

- **Recurring wins:**
  - Every key stays in its one hashed bucket, with SLRU shared across the whole bucket.
  - Giving the protected segment a bigger share kept raising hit rate, but each step gained less and the latest steps cost tail latency.
  - The changes that held up all removed work: one hash per operation, a smaller 16-byte item, and a tag array checked before full key compares.
  - Past evictions ("ghost history") help only when they decide where a new key enters probation.
- **Recurring failures:**
  - Placing a key outside its hashed bucket always fails the slot-legality check.
  - Anything that shrinks or displaces the protected segment loses hit rate.
  - A longer ghost list added nothing.
  - The undersized frequency sketch for admission filled up and stopped telling keys apart.
  - Since the item shrank to 16 bytes, trying to move less data hasn't paid off; the extra branches cost as much as they saved.
  - Improvements were dropped when a variant started from the wrong parent version.
  - Latency sometimes moved with no code cause.
- **Open questions:**
  - Retry admission only with a signal that is checked to tell keys apart first.
  - Cut fixed instructions on the get-hit path.
  - Check whether the 15/16 protected share is worth its tail cost.
  - Plain LRU against SLRU on the same base is still untested; it has been carried forward since cycle 0002.
  - Try a ghost lookup that doesn't scan the list linearly.
  - Check how the protected/probation split behaves at different bucket sizes.
- **Per-policy notes:** short notes on split-half LRU, plain LRU, SLRU, ghost-guided placement, the ARC-style adaptive split, and frequency-based admission.


---

- finished: 2026-10-20T09:56:33+00:00
- duration: 31.10s
- status: ok
- input_tokens: 4
- output_tokens: 2886
- cache_read_tokens: 42176
- cache_write_tokens: 15736
- cost_usd: 0.192059
