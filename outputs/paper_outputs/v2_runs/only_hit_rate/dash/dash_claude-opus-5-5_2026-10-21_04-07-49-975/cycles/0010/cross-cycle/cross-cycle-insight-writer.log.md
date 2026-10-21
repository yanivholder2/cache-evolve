# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-21T05:17:32+00:00

## timeline

### 05:17:56 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0010)\n\n## Recurring wins\n\n- **Filter entry into protected space instead of adding protection.** This held from 0001 to 0009. The winners made the normal tier and the stash MRU end harder to reach: stricter stash→target admission in 0001, LRU-end landing for fresh keys in 0004, rarer and then no BIP in 0006 and 0008, and a recency gate on ghost hits in 0007. Protected stash capacity is close to zero-sum, so every slot taken from blind or stale entries goes to residents with evidence behind them.\n- **Fresh keys face a harsh bar; proven ex-residents get a cheap second chance.** This is SLRU/2Q-style behaviour. Demoted normal-tier victims re-enter the stash at MRU with a pre-credit, which has paid off since 0002. Cycle 0010 showed this route must not be filtered.\n- **Placement is the lever, not counter credit.** Changing where a key lands in the stash moved hit-rate in 0004, 0005 and 0006. Changing promotion counters or credits did nothing in 0004.\n- **For ghost evidence, recency beats volume.** A ghost about one cache in size whose hit changes placement helped (0005). A 4× window hurt (0006). A gate at roughly half a window helped (0007). Shrinking the table to ½ and then ¼ cost no hits (0009–0010), because the age gate sets the useful horizon, not table size.\n- **Simplifying wins are real wins.** Removing BIP, halving the ghost, and quartering it each held or improved hit-rate while cutting state and branches. Removing mechanisms that duplicate others has paid repeatedly.\n- **Orthogonal filters stack.** The ghost recency gate (selective path) and BIP removal (blind path) worked better together than either alone (0008).\n\n## Recurring failures\n\n- **Anything that takes protected capacity from ex-residents loses.** This covers wider probation (0005), set-dueling DIP (0005), the larger ghost window (0006), and filtering untouched demotions down to probation (0010, the largest loss in the series).\n- **Stacking retention on items already protected adds nothing.** Examples: the threshold of 3 combined with MRU re-entry, the CLOCK scan (0003), scan protection for ghost-admitted keys (0006), and the fresh-key doorkeeper (0009). MRU placement plus the second-chance scan already cover these items.\n- **Counter-only tweaks fail.** Ghost pre-credit (0004), the TinyLFU-style frequency duel (0004) and the target→probing frequency bar (0002) all came out flat. Keys with no resident hit never reach the bar, and normal-tier frequency is coupled to position.\n- **Rerouting between tiers is near zero-sum.** Free-slot spill (0001), cascade-free routing (0003, 0007), and sending recent ghost hits straight to target (0010) all fail for the same reason: a freed slot is filled again at once, or the branch rarely fires.\n- **Fine-tuning a plateau gives noise.** Sweeping the gate cutoff, graded mid-stash placement (0008) and shifting the stash/normal ratio (0009) all landed inside single-trace noise.\n- **Efficiency hypotheses can't be scored.** Per-op-time and metadata-size are not fitness dimensions, so cascade-free routing produced bit-identical hit-rate and no signal (0007). Ghost-shrink \"wins\" rest on metadata savings nobody measured.\n\n## Open questions\n\n- **Evaluation coverage (unresolved for 10 cycles).** Metadata-size, per-op-time and branch-fire or displacement counters are still not measured. Every result is one LIRS trace at one cache size. It is unknown whether removing BIP, the tight ghost horizon and LRU-landing of fresh keys overfit LIRS loops.\n- **Ghost-size floor.** Where does the direct-mapped ghost start losing hits (⅛, 1/16)? Is the limit collisions or the epoch gate?\n- **Possible no-ops.** Several nulls may be variants whose change never reached the build or never fired: 0003 v001, 0007 v003 and 0010 v003. Writers have never been able to inspect overlay source.\n- **Remaining hit-rate headroom.** Most remaining misses look like true one-touch keys or reuse beyond about one cache. Can any mechanism recover these, or does that need a second workload to show up?\n\n## Per-policy notes\n\n- **Normal tier (target/probing buckets):** Front landing already filters the probing edge, so admission gates there are inert. Eviction triggers cross-bucket demotion cascades, but MRU stash re-entry made their hit-rate cost negligible. Moving slots between tiers is roughly neutral at the current split.\n- **Stash (victim cache):** It works as a probation slot at the LRU end plus a protected region for demoted ex-residents and recent ghost hits. The stash→target threshold sits near its optimum at 2–3. Victim choice (second-chance scan) and insert position within the stash are tuned out.\n- **Ghost table:** Use it only to steer placement, never to add counter credit. A recency gate of about half a window is essential. A ¼-size direct-mapped table is lossless so far. Extra protection for ghost-admitted keys adds nothing.\n- **BIP/DIP insertion:** BIP helped only until a selective ghost existed. After that the best setting was off, and adaptive DIP lost. Don't reintroduce either on this trace.\n- **Current lineage:** 0004 v003 → 0005 v003 → 0006 v003 → 0007 v002 → 0008 v001 → 0009 v002 → 0010 v001. Current state: recency-gated ¼-size ghost, no BIP, LRU-landing fresh keys, MRU stash re-entry for demotions.\n"
}
```

### 05:18:04 chat

I wrote `cross-cycle-insights.md` in the cycle 0010 cross-cycle folder, with the four sections in the required order, built only from the ledger digest.

- **Recurring wins:**
  - Tighter rules on what gets into protected space keep paying off, rather than extra protection. Examples are stricter admission and fresh keys landing at the LRU end of the stash.
  - Items pushed out of the normal tier should keep re-entering the stash at the MRU end. Cycle 0010's heaviest loss came from filtering that route.
  - Where an item lands matters; changing promotion counters or credits does not.
  - Recent ghost hits matter more than a large ghost table. A quarter-size ghost lost no hits.
  - Removing redundant mechanisms (BIP, extra ghost size) held hit-rate while cutting state.
- **Recurring failures:**
  - Taking protected capacity away from ex-residents.
  - Stacking protection on items that are already protected.
  - Counter-only tweaks.
  - Rerouting items between tiers, which is close to zero-sum.
  - Fine-tuning settings that have already plateaued.
  - Efficiency ideas, which can't be scored because per-op-time and metadata-size aren't measured.
- **Open questions:**
  - For ten cycles, evaluation has covered only hit-rate, on one LIRS trace at one cache size.
  - How far the ghost table can shrink before it starts losing hits.
  - Several flat results may be changes that never reached the build or never ran.
  - Whether any hit-rate headroom is left on this trace.
- **Per-policy notes:** grouped by normal tier, stash, ghost table and BIP/DIP insertion. I also added the winning lineage from cycle 0004 to 0010 and the design it has arrived at.


---

- finished: 2026-10-21T05:18:04+00:00
- duration: 32.67s
- status: ok
- input_tokens: 4
- output_tokens: 2726
- cache_read_tokens: 42336
- cache_write_tokens: 15786
- cost_usd: 0.190375
