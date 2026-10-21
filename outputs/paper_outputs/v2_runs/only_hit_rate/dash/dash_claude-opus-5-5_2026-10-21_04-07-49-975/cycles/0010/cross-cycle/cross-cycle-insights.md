# Cross-Cycle Insights (through cycle 0010)

## Recurring wins

- **Filter entry into protected space instead of adding protection.** This held from 0001 to 0009. The winners made the normal tier and the stash MRU end harder to reach: stricter stash→target admission in 0001, LRU-end landing for fresh keys in 0004, rarer and then no BIP in 0006 and 0008, and a recency gate on ghost hits in 0007. Protected stash capacity is close to zero-sum, so every slot taken from blind or stale entries goes to residents with evidence behind them.
- **Fresh keys face a harsh bar; proven ex-residents get a cheap second chance.** This is SLRU/2Q-style behaviour. Demoted normal-tier victims re-enter the stash at MRU with a pre-credit, which has paid off since 0002. Cycle 0010 showed this route must not be filtered.
- **Placement is the lever, not counter credit.** Changing where a key lands in the stash moved hit-rate in 0004, 0005 and 0006. Changing promotion counters or credits did nothing in 0004.
- **For ghost evidence, recency beats volume.** A ghost about one cache in size whose hit changes placement helped (0005). A 4× window hurt (0006). A gate at roughly half a window helped (0007). Shrinking the table to ½ and then ¼ cost no hits (0009–0010), because the age gate sets the useful horizon, not table size.
- **Simplifying wins are real wins.** Removing BIP, halving the ghost, and quartering it each held or improved hit-rate while cutting state and branches. Removing mechanisms that duplicate others has paid repeatedly.
- **Orthogonal filters stack.** The ghost recency gate (selective path) and BIP removal (blind path) worked better together than either alone (0008).

## Recurring failures

- **Anything that takes protected capacity from ex-residents loses.** This covers wider probation (0005), set-dueling DIP (0005), the larger ghost window (0006), and filtering untouched demotions down to probation (0010, the largest loss in the series).
- **Stacking retention on items already protected adds nothing.** Examples: the threshold of 3 combined with MRU re-entry, the CLOCK scan (0003), scan protection for ghost-admitted keys (0006), and the fresh-key doorkeeper (0009). MRU placement plus the second-chance scan already cover these items.
- **Counter-only tweaks fail.** Ghost pre-credit (0004), the TinyLFU-style frequency duel (0004) and the target→probing frequency bar (0002) all came out flat. Keys with no resident hit never reach the bar, and normal-tier frequency is coupled to position.
- **Rerouting between tiers is near zero-sum.** Free-slot spill (0001), cascade-free routing (0003, 0007), and sending recent ghost hits straight to target (0010) all fail for the same reason: a freed slot is filled again at once, or the branch rarely fires.
- **Fine-tuning a plateau gives noise.** Sweeping the gate cutoff, graded mid-stash placement (0008) and shifting the stash/normal ratio (0009) all landed inside single-trace noise.
- **Efficiency hypotheses can't be scored.** Per-op-time and metadata-size are not fitness dimensions, so cascade-free routing produced bit-identical hit-rate and no signal (0007). Ghost-shrink "wins" rest on metadata savings nobody measured.

## Open questions

- **Evaluation coverage (unresolved for 10 cycles).** Metadata-size, per-op-time and branch-fire or displacement counters are still not measured. Every result is one LIRS trace at one cache size. It is unknown whether removing BIP, the tight ghost horizon and LRU-landing of fresh keys overfit LIRS loops.
- **Ghost-size floor.** Where does the direct-mapped ghost start losing hits (⅛, 1/16)? Is the limit collisions or the epoch gate?
- **Possible no-ops.** Several nulls may be variants whose change never reached the build or never fired: 0003 v001, 0007 v003 and 0010 v003. Writers have never been able to inspect overlay source.
- **Remaining hit-rate headroom.** Most remaining misses look like true one-touch keys or reuse beyond about one cache. Can any mechanism recover these, or does that need a second workload to show up?

## Per-policy notes

- **Normal tier (target/probing buckets):** Front landing already filters the probing edge, so admission gates there are inert. Eviction triggers cross-bucket demotion cascades, but MRU stash re-entry made their hit-rate cost negligible. Moving slots between tiers is roughly neutral at the current split.
- **Stash (victim cache):** It works as a probation slot at the LRU end plus a protected region for demoted ex-residents and recent ghost hits. The stash→target threshold sits near its optimum at 2–3. Victim choice (second-chance scan) and insert position within the stash are tuned out.
- **Ghost table:** Use it only to steer placement, never to add counter credit. A recency gate of about half a window is essential. A ¼-size direct-mapped table is lossless so far. Extra protection for ghost-admitted keys adds nothing.
- **BIP/DIP insertion:** BIP helped only until a selective ghost existed. After that the best setting was off, and adaptive DIP lost. Don't reintroduce either on this trace.
- **Current lineage:** 0004 v003 → 0005 v003 → 0006 v003 → 0007 v002 → 0008 v001 → 0009 v002 → 0010 v001. Current state: recency-gated ¼-size ghost, no BIP, LRU-landing fresh keys, MRU stash re-entry for demotions.
