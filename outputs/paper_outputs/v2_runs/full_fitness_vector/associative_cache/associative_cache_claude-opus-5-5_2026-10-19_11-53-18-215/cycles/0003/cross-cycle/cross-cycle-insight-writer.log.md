# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-19T12:19:51+00:00

## timeline

### 12:20:07 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **One shared policy per merged bucket.** This has been the base since cycle 0001 (v003) and has paid off in every cycle since. Each key hashes once to one physical bucket, and the bucket uses one shared policy. This passes P2 slot_legality and gives the best hit rates seen so far.\n- **Cheaper mechanics give speedups that carry across policies.** One hash per op, contiguous bucket storage and in-place policy updates helped in cycle 0001. Narrower items and shorter moves (cycles 0002 and 0003) also helped. These changes made get and put faster without changing hit rate, so they can be stacked freely.\n- **Latency comes from data movement.** Each cycle found the same cause: memmoves of whole items across the bucket. Three things kept helping:\n  - smaller `Item`s (the 24-byte logical-tick form)\n  - scanning from the MRU end\n  - keeping eviction rotates inside probation\n- **Smaller metadata costs nothing under position-ordered policies.** Under LRU and SLRU the `timestamp` and `lfu_counter` fields are never read. Shrinking them made metadata smaller and put faster, with hit rate flat.\n- **No wall-clock reads on the hot path.** Replacing `Instant::now()` with logical ticks made put and the tails faster.\n- **Scan resistance adds hit rate on this LIRS-style trace.** SLRU beat its parent on hit rate. It needs no extra per-item metadata because the segment boundary is stored in the existing length field.\n\n## Recurring failures\n\n- **Placing a key outside its hashed bucket.** Two-choice, cuckoo and cross-bucket victim designs failed P2 slot_legality on every attempt. Do not propose them again.\n- **Splitting LRU into home and partner halves inside a bucket.** It lowered hit rate and added tail cost on the partner path. Removing the split recovered hit rate in cycle 0002.\n- **Rewarding a ghost hit with protected status.** In cycle 0003, sending returning ghost keys straight to protected broke scan resistance and lost hit rate. Protected is usually full, so each such insert pushes out an item that earned its place with a real second hit. A ghost hit is weaker evidence than a resident second hit.\n- **Trusting small latency changes without a clear cause.** Some get-median and per-op shifts appeared with no explanation in the code and are probably noise. Re-measure them before giving any design credit.\n\n## Open questions\n\n- **Stacking the cycle 0003 wins.** Combining the 24-byte `Item` with MRU-first `find` and the probation-only rotate has not been tested yet. They should add up.\n- **Removing the leftover protected-wide moves.** Two moves still cross all of protected:\n  - the re-find after insert, which could be avoided by returning the index from put\n  - the rotate on a probation-to-protected promotion\n  \n  Ring offsets or swap-based promotion might remove both.\n- **Plain LRU vs SLRU on the same base.** This has been deferred for two cycles. So has tuning the 3/4 protected fraction.\n- **Probation-only use of history.** Can ghost history help hit rate if it only affects probation or an ARC-style adaptive split, and never protected?\n- **A tag or fingerprint array scanned before full items.** Would it lower the get median further?\n- **Removing `timestamp` and `lfu_counter` entirely on the SLRU path.** Would this shrink metadata further without hurting other policy kinds?\n\n## Per-policy notes\n\n- **Split-half LRU (home/partner):** Superseded. It lowered hit rate and added tail cost on the partner path.\n- **Plain LRU over a merged bucket:** It recovered hit rate above baseline in cycle 0002. It has never been compared directly with SLRU on a matching base.\n- **SLRU (probation/protected, 3/4 protected):** The current best policy and the current base. It gains hit rate from scan resistance. Its cost is in moves that touch protected, so keep evictions inside probation. The protected fraction has not been tuned.\n- **SLRU + ghost-to-protected:** Lowered hit rate, so drop it. Ghost signals should only influence probation or an adaptive split.\n- **Timestamp / LFU-based policies:** None have been tried inside the single-bucket rule. If one is tried, it would need the metadata fields that LRU and SLRU leave unused, so check whether it can still use the compact `Item`.\n"
}
```

### 12:20:14 chat

I wrote `cross-cycle-insights.md` from the three cycle digests, using the four required sections in order. It gives no numbers, only qualitative statements.

- **Recurring wins:**
  - The merged bucket with one shared policy has been the base since cycle 0001's v003.
  - Wherever latency was traced, the cause was moving whole items around inside the bucket. Smaller items, scanning from the most-recent end and keeping evictions inside probation all cut it.
  - Shrinking unused metadata cost no hit rate.
  - Replacing wall-clock reads with logical ticks made put and the slow tail faster.
  - SLRU's scan resistance raised hit rate on this trace.
- **Recurring failures:**
  - Any design that places a key outside its hashed bucket fails the slot_legality check every time.
  - Splitting each bucket's LRU into halves lowered hit rate.
  - Sending returning keys straight to protected broke scan resistance and lost hit rate.
  - Some small latency changes had no cause in the code and need re-measuring before they get credit.
- **Open questions:**
  - Cycle 0003's speed wins have not been combined yet.
  - Two moves still walk all of protected: the re-find after insert and the rotate on promotion.
  - Plain LRU against SLRU on the same base, and tuning the protected fraction, have been put off for two cycles.
  - Still untested: history that only affects probation, a tag array scanned first to speed up gets, and removing the unused fields entirely.
- **Per-policy notes:** separate entries for split-half LRU, plain LRU, SLRU, SLRU with the protected-admission change, and timestamp/frequency policies, which have not been tried yet.


---

- finished: 2026-10-19T12:20:14+00:00
- duration: 23.48s
- status: ok
- input_tokens: 4
- output_tokens: 2207
- cache_read_tokens: 35764
- cache_write_tokens: 8730
- cost_usd: 0.121149
