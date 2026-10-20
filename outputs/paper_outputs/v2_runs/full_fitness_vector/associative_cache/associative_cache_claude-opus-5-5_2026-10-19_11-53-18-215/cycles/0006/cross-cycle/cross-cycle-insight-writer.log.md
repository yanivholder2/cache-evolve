# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-19T14:10:58+00:00

## timeline

### 14:11:22 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **One hashed bucket per key, with one shared policy over a merged bucket.** This has been the base since cycle 0002. Designs that place keys in several buckets cannot pass P2 slot_legality. Split per-half policies held hit rate down until they were merged.\n- **Cutting data movement is the main latency lever.** Cost is mostly memmove of items inside the bucket. Several fixes made get and put faster without changing hit rate, and they stack:\n  - narrower `Item` strides: 40 → 24 → 16 bytes `{ key, value }`\n  - evicting only from probation, with short rotates\n  - returning the insert index instead of re-finding it\n  - The biggest gains show up in the tails, where the longest shifts happen.\n- **Removing unused metadata is free.** Under position-ordered policies, timestamp and LFU fields are never read. Removing them brought metadata-size to its floor with no hit-rate cost. Taking wall-clock reads off the hot path helped put latency and tails.\n- **SLRU scan resistance pays on this trace, and more protected capacity raises hit rate.** The cap went 3/4 → 7/8. From cycle 0006 on, this costs get and put latency (see Per-policy notes).\n- **Ghost history helps only when it decides where a key enters probation.** Cold keys go mid-probation and ghost hits go to MRU. Used this way it gained hit rate without displacing protected items.\n- **Changing one axis per variant from a shared parent works well.** Each Winner isolates one lever: get-median, hit-rate, or metadata/tails. That makes merging predictable.\n\n## Recurring failures\n\n- **Cross-bucket placement** (two-choice, cuckoo, victim areas) fails P2 slot_legality every time. Do not propose it.\n- **Anything that shrinks or displaces protected loses hit rate:**\n  - ghost-to-protected admission\n  - ARC-style adaptive protected caps, which drift toward LRU because the shrink signal dominates and grow steps round to zero in small buckets\n  - split home/partner halves\n- **Skipping shallow moves**, such as lazy promotion near MRU, saves nothing, because hot keys already sit near MRU. The real cost is in deep protected hits and probation promotions.\n- **Scan direction that does not match the segment layout.** MRU-first scans make every probation hit walk all of protected first, which hurt the get median. Credit that was once given to scan order belonged to the shorter moves.\n- **Enlarging the ghost ring.** Half a bucket already captures the usable reuse. A longer ring adds false positives and makes the linear O(ring) scan longer on every miss-path put.\n- **Fragile carry-forward.** Improvements were lost or not merged several times: the 24-byte item, the 16-byte item, and the tag-first `find` from cycle 0005, which is still unmerged. Some regressions came from a missing carry-forward, not from the idea being tested.\n- **Latency shifts with no code cause** show up almost every cycle. Small per-op deltas are unreliable until re-measured side by side with the parent.\n\n## Open questions\n\n- Can ring offsets or swap-based promotion make deep protected and probation moves cheap enough to keep 7/8's hit rate without its latency cost?\n- On the 16-byte item with a 7/8 cap, how much does keeping the tag array in sync cost on put, compared with the get-median gain and the removed second hash?\n- Can a non-linear ghost lookup (a bitset or a hashed slot) with a fingerprint taken from the existing hash shrink the put tail on the miss path?\n- Where is the best point on the hit-rate/latency curve for the protected cap: between 3/4 and 7/8, or above 7/8 on large buckets? How sensitive is the probation floor to bucket size?\n- Plain LRU vs SLRU has never been measured head to head on the same base. It has been carried forward unaddressed since cycle 0002.\n- Is mid-probation the best entry point for cold keys?\n\n## Per-policy notes\n\n- **Split-half LRU (home/partner):** worse hit rate than one shared policy, and the partner path adds tail latency. Retired.\n- **Plain LRU over a merged bucket:** recovered hit rate above the original baseline in cycle 0002. It has not been tested against SLRU since, and is likely dominated by SLRU on this LIRS-style trace.\n- **SLRU (probation/protected, boundary stored in `len0`):** the current base.\n  - Hit rate rises with protected share.\n  - Latency is set by how long rotates are. Larger protected means longer protected-hit rotates. A 1-slot probation means more frequent whole-bucket shifts on eviction and ghost re-insert.\n  - Evict from probation only, and keep moves out of protected wherever possible.\n- **SLRU with ghost ring:** helps only through probation placement.\n  - Using it for protected admission or adaptive cap sizing hurts hit rate.\n  - Its put-tail cost (second hash, linear ring scan, ring writes) is the main overhead left to reduce.\n  - Its per-bucket memory does not show up in the metadata-size dimension.\n- **ARC-style adaptive split:** lost hit rate. Retired.\n"
}
```

### 14:11:29 chat

I wrote `cross-cycle-insights.md` in the cycle 0006 cross-cycle output dir. It uses the four required sections and draws only on the six-cycle ledger.

**Recurring wins**
- Every key stays in its one hashed bucket, with one shared SLRU policy over the merged bucket.
- Latency comes mostly from moving items inside the bucket. These fixes each helped and add up:
  - narrower items: 40 → 24 → 16 bytes
  - evicting only from probation, with short rotates
  - returning the insert index instead of searching again
- Removing unused metadata fields cost nothing.
- Ghost history helps only when it decides where a new key enters probation.
- Giving protected more room raises hit rate. From cycle 0006 on, it also makes get and put slower.

**Recurring failures**
- Placing a key outside its hashed bucket always fails P2 slot_legality.
- Anything that shrinks or pushes items out of protected loses hit rate: ghost-to-protected admission, the ARC-style adaptive cap, and split halves.
- Skipping rotates near the MRU end saves nothing.
- A ghost ring longer than half a bucket doesn't help.
- Improvements keep getting lost between cycles. The tag-first `find` from cycle 0005 is still not merged.
- Latency changes that the code doesn't explain show up in most cycles.

**Open questions**
- Can ring- or swap-based moves keep the 7/8 cap's hit rate without its latency cost?
- What does keeping the tag array in sync cost on put, now that items are 16 bytes?
- Can the ghost lookup avoid its linear scan?
- Where is the best protected cap for hit rate versus latency?
- Plain LRU vs SLRU has been left open since cycle 0002 and has still never been run side by side.

The per-policy notes cover split-half LRU, plain LRU, SLRU, SLRU with the ghost ring, and the ARC-style split, and say which are retired.


---

- finished: 2026-10-19T14:11:29+00:00
- duration: 31.26s
- status: ok
- input_tokens: 4
- output_tokens: 2560
- cache_read_tokens: 39068
- cache_write_tokens: 12292
- cost_usd: 0.157366
