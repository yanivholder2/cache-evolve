# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-19T14:02:42+00:00

## timeline

### 14:03:02 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0005)\n\n## Recurring wins\n\n- **One hashed bucket per key, with one shared policy over the merged bucket.** Since cycle 0001, every Winner has kept each key in its single hashed bucket. Starting in cycle 0002, every Winner has also used one policy across the whole merged bucket.\n- **Cutting data movement is the most reliable latency lever (cycles 0002–0005).** Cost is dominated by memmove, so these all helped:\n  - narrower `Item` stride: 40 → 24 → 16 bytes\n  - shorter rotates\n  - eviction confined to probation\n  - returning the insert index instead of re-finding\n  \n  None of them changed hit rate. The largest gains showed up at the tails, where the longest shifts happen.\n- **Shrinking metadata is free under position-ordered policies.** Timestamp and LFU fields were never read under LRU/SLRU. Replacing wall-clock reads with logical ticks, then removing the fields entirely, improved metadata-size and put with hit rate flat.\n- **Giving protected more room raises hit rate.** This held across cycles: scan resistance (SLRU) beat split-half and plain designs, and raising the fixed protected fraction from 3/4 to 7/8 gained more.\n- **Ghost history helps only when it decides where a new key enters probation** (cycles 0004–0005). In that role, cold keys enter mid-probation and ghost hits enter at MRU.\n- **Changes to separate axes stack cleanly.** Hit rate (protected capacity, placement), get median (tag-first scan), and metadata/tails (stride) have each moved on their own. Merging Winners from different axes has worked every cycle.\n\n## Recurring failures\n\n- **Placing a key outside its hashed bucket** (two-choice, cuckoo, cross-bucket victim areas). These always fail the P2 slot_legality invariant.\n- **Split home/partner LRU halves.** The bookkeeping cancelled the extra flexibility and hit rate fell below baseline.\n- **Anything that shrinks or displaces protected.** Two attempts failed:\n  - ghost-to-protected admission (cycle 0003) demoted items that had earned their place with a real second hit\n  - ARC-style adaptive protected sizing (cycle 0004) drifted toward LRU, because its grow signal rounds to zero in small buckets\n  \n  Both lost scan resistance on this LIRS-like trace.\n- **Lookup order that doesn't match the segment layout.** Scanning from the MRU end forces every probation hit to walk all of protected first, which hurt the get median. Earlier credit given to \"MRU-first\" scanning actually belonged to the shorter moves.\n- **Ghost machinery has put-tail costs:** a second hash, a linear ring scan on every miss, and ring writes on every eviction. Its per-bucket memory is also invisible to the metadata-size dimension.\n- **Fragile carry-forward.** Merges have silently dropped earlier wins (for example, a shrunken `item.rs`), which then shows up as false regressions.\n- **Latency shifts with no code cause.** These have recurred (cycles 0003, 0004, 0005). Small per-op shifts are not trustworthy without a side-by-side re-run.\n\n## Open questions\n\n- On the 16-byte `Item`, does keeping the tag array in sync on put cost more than the single hash saves?\n- Can ring offsets or swap-based promotion remove the moves that span all of protected (protected-hit rotates, probation→protected promotion, ghost-hit / `len0 == 0` shifts)? These moves get longer as protected grows.\n- Has hit rate peaked at 7/8? Would 15/16 help large buckets? How does the probation floor interact with bucket size and with mid-probation insertion?\n- Can the ghost fingerprint come from the existing hash, and can ring lookup avoid a linear scan? Would a larger ring catch longer reuse distances? Is mid-probation the best entry point for cold keys?\n- Plain LRU vs SLRU on the same base is still untested after five cycles.\n- Was cycle 0005 v002's slowdown real, or noise?\n\n## Per-policy notes\n\n- **Split-half LRU (home/partner):** fails on hit rate. Its early speedup came from how it was built (one hash per op, contiguous bucket storage, in-place policy updates), not from the policy.\n- **Plain LRU over the merged bucket:** recovered hit rate above baseline. It has never been compared directly with SLRU.\n- **SLRU (probation/protected, boundary stored in `len0`):** the current lineage. Scan resistance with no per-item metadata. Its cost is movement, not the policy. Keep evictions inside probation and keep protected large (7/8 currently best).\n- **SLRU + ghost ring:** placement-only use gains hit rate. Using it for protected admission or protected sizing loses hit rate. It costs put tail unless made cheaper.\n- **ARC-style adaptive split:** loses on this trace and bucket geometry. Drop it.\n- **Timestamp/LFU-based policies:** not pursued. Their fields have been removed from `Item`, so reviving them would cost metadata-size.\n"
}
```

### 14:03:09 chat

I wrote `cross-cycle-insights.md` from the cycle 0001–0005 ledger, using the four required sections in order and no numbers beyond what the ledger states.

- **Recurring wins:**
  - Every key stays in its one hashed bucket, and from cycle 0002 on there is one shared policy across the merged bucket.
  - Moving less data is the steadiest way to cut latency: a narrower item size, shorter shifts, evicting only from probation, and returning the insert index. None of these changed hit rate.
  - Removing metadata that isn't read was free.
  - Giving the protected segment more room raises hit rate.
  - Ghost history helps only when it decides where a new key enters probation.
  - Changes along separate dimensions combine cleanly.
- **Recurring failures:**
  - Placing a key outside its hashed bucket always fails the slot_legality invariant.
  - Split home/partner halves lost hit rate.
  - Anything that shrinks or displaces protected lost hit rate. That covers ghost-to-protected admission and adaptive protected sizing.
  - The scan order has to match the segment layout, or the get median suffers.
  - The ghost ring costs put tail.
  - Merges sometimes silently drop earlier gains, which then look like regressions.
  - Latency moved in three cycles with no code cause.
- **Open questions:**
  - Whether keeping the tag array in sync costs more on put than the single hash saves.
  - Cheaper moves within protected.
  - Protected fractions above 7/8 and sensitivity to bucket size.
  - A cheaper ghost path.
  - Plain LRU vs SLRU, still untested after five cycles.
  - Whether cycle 0005 v002's slowdown was real.
- **Per-policy notes:** short entries for split-half LRU, plain LRU, SLRU (the current line), SLRU with a ghost ring, adaptive protected sizing (drop it), and timestamp/LFU policies (not pursued).


---

- finished: 2026-10-19T14:03:09+00:00
- duration: 27.23s
- status: ok
- input_tokens: 4
- output_tokens: 2557
- cache_read_tokens: 37958
- cache_write_tokens: 11182
- cost_usd: 0.148204
