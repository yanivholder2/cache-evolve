# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-19T12:27:13+00:00

## timeline

### 12:27:29 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Merged single bucket with one shared policy (cycles 1–4).** A key lives only in its one hashed bucket, and the whole bucket uses one policy. This is the only shape that passes P2 slot_legality, and it has been the base of every Winner since cycle 1.\n- **Cutting data movement makes get and put faster without changing hit rate (cycles 1–4).** The changes that helped: hash once per op, contiguous bucket storage, in-place policy updates, a narrower 24-byte `Item`, evicting only from probation with short rotates, and returning the insert index instead of re-finding. They stacked every time they were combined. Moves that cross protected are the expensive ones.\n- **Taking dead or costly fields off the hot path is free (cycles 2–3).** Replacing the wall-clock timestamp with logical ticks made put faster. Under position-ordered policies, `timestamp`/`lfu_counter` are never read. Removing them shrank metadata at no hit-rate cost.\n- **Scan resistance pays on this LIRS-like trace (cycles 2–4).** SLRU with a strong protected segment beat both split-half LRU and its own parent on hit rate.\n- **Ghost history helps when it only decides placement inside probation (cycle 4).** Ghost hits enter at MRU and cold keys enter mid-probation. This gained hit rate and never displaces protected.\n\n## Recurring failures\n\n- **Putting keys outside their hashed bucket (cycle 1).** Two-choice, cuckoo and cross-bucket victim areas all fail P2 slot_legality every time. This direction is closed.\n- **Splitting a bucket into home/partner halves (cycles 1–2).** The bookkeeping cancelled the extra flexibility, hurt hit rate and added a partner-path tail.\n- **Anything that weakens or displaces protected (cycles 3–4).** Ghost-to-protected admission and an ARC-style adaptive protected cap both lost hit rate. The cap drifted toward plain LRU: ghost hits shrank it often, and its grow signal rounds to zero in small buckets.\n- **Lookup order that ignores the segment layout (cycles 2, 4).** Full-bucket scans and MRU-end scans that walk all of protected before reaching probation keep pushing up the get median.\n- **Giving credit to unexplained latency shifts (cycles 3–4).** Small changes in per-op time with no code cause showed up and later went away. Re-measure before keeping a change for latency alone.\n\n## Open questions\n\n- **Segment-aware or forward `find`.** Does either recover the get median while keeping the stacked put gains?\n- **Cheaper ghost path.** Can the fingerprint come from the existing hash, with no second hash? Can the ring be checked without a linear scan? Can the ghost-hit and `len0 == 0` branches use the short-rotate eviction? Each would reduce ghost's put-tail cost.\n- **Cold-entry position and ghost-ring size.** Is mid-probation the best entry point for cold keys? Would a larger ring catch longer reuse distances? Note that ring memory is not counted in the metadata-size dimension.\n- **Plain LRU vs SLRU, and fixed protected fractions other than 3/4.** Still untested on a common base after four cycles.\n- **Remaining protected-wide moves.** Can ring offsets or swap-based promotion remove the rotate across protected on a probation hit?\n\n## Per-policy notes\n\n- **Split-half LRU (home/partner):** Dropped. It held hit rate down and added a tail.\n- **Plain LRU over merged bucket:** It lifted hit rate above baseline, but it has never been compared with SLRU directly. It also lacks scan resistance, which this trace rewards.\n- **SLRU (probation + protected, 3/4 protected):** Current base policy. Segment membership lives in the `len0` boundary, so it adds no per-item metadata. Its cost comes from movement, not from the policy. Keep evictions and rotates inside probation.\n- **SLRU + ghost placement:** Placement-only use is the current best hit-rate lever. It costs a second hash, a ring scan on every miss and a ring write on every eviction, all on the put tail.\n- **SLRU + ghost → protected / adaptive cap (ARC-like):** Both lost hit rate. A ghost hit is weaker evidence than a resident second hit, so it should not shrink or invade protected.\n"
}
```

### 12:27:37 thinking



### 12:27:37 chat

I wrote `cross-cycle-insights.md` in the cycle 0004 cross-cycle folder. It covers cycles 0001–0004 under the four required sections, in order. It uses only the ledger and quotes no numbers.

- **Recurring wins:**
  - Every Winner since cycle 1 keeps each key in its one hashed bucket under a single policy. That is the only design that has passed the P2 slot_legality check.
  - Changes that cut data movement made get and put faster without hurting hit rate, and they kept adding up. Examples: hashing once, a smaller 24-byte item record, evicting only from probation, and returning the insert position instead of searching again.
  - Removing timestamp and frequency fields that are never read cost nothing.
  - SLRU's scan resistance helps on this trace.
  - Ghost history helps when it only decides where a new key goes inside probation.
- **Recurring failures:**
  - Placing keys outside their hashed bucket always fails the P2 check.
  - Splitting buckets into home and partner halves held hit rate down.
  - Anything that shrinks or pushes into protected lost hit rate, including both ghost-into-protected and the adaptive protected size.
  - Lookup orders that walk all of protected first keep slowing the median get.
  - Latency changes with no cause in the code later disappeared, so they need re-measuring before they get credit.
- **Open questions:**
  - Can a segment-aware or forward lookup bring the median get back down?
  - Can the ghost path get cheaper (reuse the existing hash, avoid scanning the ring)?
  - Where should cold keys enter, and how big should the ghost ring be?
  - Plain LRU vs SLRU and protected fractions other than 3/4 have been carried forward since cycle 2 and never tested.
  - Can the remaining moves across protected be removed?
- **Per-policy notes:** separate entries for split-half LRU, plain LRU, SLRU (the current base), SLRU with ghost placement, and SLRU with ghost-into-protected or an adaptive cap.


---

- finished: 2026-10-19T12:27:37+00:00
- duration: 23.67s
- status: ok
- input_tokens: 4
- output_tokens: 2337
- cache_read_tokens: 36908
- cache_write_tokens: 9867
- cost_usd: 0.133074
