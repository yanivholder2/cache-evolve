# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-19T14:19:53+00:00

## timeline

### 14:20:12 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Merged single bucket, one shared policy.** Every key lives in its one hashed bucket (required by P2 slot_legality), and the whole bucket runs a single replacement policy. This has been the base since cycle 0002. Split-half designs held hit rate down.\n- **Cost is mostly data movement, so a narrower stride helps.** Shrinking `Item` (logical ticks in 0002/0003, then 16-byte `{ key, value }` in 0005) made get and put faster every time, with hit rate unchanged. The biggest gains were at the tails. Metadata-size is now at its floor.\n- **Shorter put-side moves stack.** Evicting only from probation, using short rotates, and returning the insert index instead of re-finding each made put faster over cycles 0003–0004, and SLRU semantics stayed the same.\n- **Lookup tuned separately from replacement.** One hash per op (0001), then a tag-first forward scan using stored tags (0005, merged in 0007), cut get and put latency without changing any replacement decision. Tag upkeep costs less than the hashes and compares it removes.\n- **Bigger protected segment means more hit rate.** Going 3/4 → 7/8 → 15/16 (on large buckets) raised hit rate each time, but each step gained less. Since 0006 it also costs latency (see below).\n- **Ghost history used only for placement.** Choosing where a new key enters probation (cold keys mid-probation, ghost hits at MRU) gained hit rate without touching protected.\n- **No wall-clock reads on the hot path.** Removing `Instant::now()` helped put and the tails.\n\n## Recurring failures\n\n- **Putting a key outside its hashed bucket** (two-choice, cuckoo, cross-bucket victim areas). This always fails P2 slot_legality.\n- **Anything that shrinks or displaces protected.** Ghost-to-protected admission (0003) and an ARC-style adaptive cap (0004) both lost hit rate. They break scan resistance on this LIRS-like trace, because a ghost hit is weaker evidence than a resident second hit.\n- **Skipping or limiting protected-side moves.** Lazy near-MRU promotion (0006) saved nothing, because hot keys already sit near MRU. Move-ahead-k (0007) demoted warm keys, caused churn, and slowed put. Protected rotates are cheap at this stride, so they are not the bottleneck.\n- **A longer ghost ring.** Half a bucket already captures the reuse that placement can use. Going longer adds false positives and lengthens the linear ring scan on every miss.\n- **Scanning from the MRU end.** This forces probation hits to scan all of protected first, which hurt the get median. Earlier gains credited to it actually came from shorter moves.\n- **Fragile carry-forward.** Improvements were repeatedly left unmerged or lost, including `item.rs` and the tag array. Variants then branched from stale parents, and the regressions this caused were misread.\n- **Unexplained latency shifts.** These came up in most cycles. Small per-op changes with no code cause should not get credit, or blame, until re-measured.\n\n## Open questions\n\n- **Where does miss-path put cost come from** now that probation is about one slot: ghost scan and writes, evictions, or whole-bucket insert/remove shifts? Would a hashed or bitset ghost lookup, or a ring-offset layout for probation only, remove it?\n- **Can admission take over as the hit-rate lever** now that the cap is flattening? For example, a frequency or ghost filter that keeps one-hit-wonders out of probation.\n- **Is a cap between 3/4 and 7/8, or a 2-slot probation floor on mid-size buckets, a better trade** between hit rate and latency? How sensitive is the result to bucket size?\n- **Is the mid-probation entry point for cold keys still meaningful** when probation is this thin?\n- **Plain LRU vs SLRU on the same base** has never been measured side by side, although LRU is expected to lose scan resistance.\n\n## Per-policy notes\n\n- **Split-half home/partner LRU:** This passed the gates only because it merged the pair into one physical bucket. Its policy logic held hit rate down. Abandoned.\n- **Plain LRU over a merged bucket:** It beat baseline hit rate in 0002, but has not been explored since and has never been compared directly with SLRU.\n- **SLRU (probation + protected), the current lineage:** Segment membership is stored in a boundary index at zero metadata cost, and the timestamp and LFU fields were dead weight. Hit rate rises with protected share. Latency is set by probation-side churn, not by protected rotates. The current base is a 16-byte item, tag-first single-hash find, short probation-only eviction, ghost-guided placement, and a 15/16 cap on large buckets.\n- **SLRU + ghost ring:** Helpful only as a placement hint inside probation. It hurts when it promotes into protected or resizes protected. Its cost is a linear ring scan and writes on the miss path, which show up on the put tail.\n- **ARC-style adaptive split:** Drifts toward LRU on this trace because shrink signals are frequent and grow signals round to zero in small buckets. Abandoned.\n"
}
```

### 14:20:18 chat

I've written `cross-cycle-insights.md` in the cycle 0007 cross-cycle directory, built fresh from the ledger digest (cycles 0001–0007) with the four sections in the given order.

**What has kept working:**
- Every key stays in its one hashed bucket, and the whole bucket runs a single shared policy.
- Shrinking the stored item made get and put faster every time it was tried, with hit rate unchanged.
- Put-side fixes that shorten data moves stacked well together.
- Hashing once and scanning tags first cut latency without changing any eviction decision.
- A bigger protected segment kept raising hit rate, but each step gained less, and since cycle 0006 it also costs latency.
- Ghost history helps only when it decides where a new key enters probation.

**What has kept failing:**
- Placing a key outside its hashed bucket always fails the slot_legality invariant.
- Anything that shrinks or displaces protected lost hit rate.
- Skipping or limiting moves inside protected saved nothing, because those moves are already cheap.
- Making the ghost ring longer, and scanning from the MRU end, both made things worse.
- Improvements were repeatedly lost between cycles because variants branched from older parents.
- Latency shifts with no code cause came up in most cycles.

**Open questions:**
- Where the miss-path put cost actually comes from now that probation is about one slot.
- Whether admission filtering can take over as the hit-rate lever as the protected cap flattens out.
- Whether a cap between 3/4 and 7/8 is a better hit-rate/latency trade.
- Plain LRU against SLRU on the same base, which no cycle has measured yet.

The per-policy notes cover split-half LRU, plain LRU, SLRU (the current lineage), SLRU with a ghost ring, and the ARC-style adaptive split.


---

- finished: 2026-10-19T14:20:18+00:00
- duration: 25.57s
- status: ok
- input_tokens: 4
- output_tokens: 2519
- cache_read_tokens: 40092
- cache_write_tokens: 13324
- cost_usd: 0.165006
