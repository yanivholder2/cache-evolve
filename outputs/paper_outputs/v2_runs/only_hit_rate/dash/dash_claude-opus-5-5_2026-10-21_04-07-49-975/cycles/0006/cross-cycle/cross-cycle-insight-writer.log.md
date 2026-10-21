# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:49:50+00:00

## timeline

### 04:50:11 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0006)\n\n## Recurring wins\n\n- **Fresh keys face a harsh bar; proven ex-residents are protected.** Winners in 0001, 0002, 0004, 0005 and 0006 all used this SLRU/2Q-style split, applied first through admission thresholds, then through stash re-entry position, BIP insertion and ghost-guided placement.\n- **Placement beats counters.** Where a key lands (MRU stash re-entry in 0002, BIP fresh insertion in 0004, ghost hits mapped to MRU placement in 0005) moved hit-rate. Credit or threshold tweaks on the same keys mostly did not.\n- **Protected stash capacity is zero-sum.** Variants that added evidence-gated admission (ghost, 0005) or removed blind admission (rarer BIP, 0006) gained. The protected region for ex-residents should only be entered on evidence.\n- **Evidence has to be timely.** Ghost history pays off with a window of about one cache's worth of evictions, and only when a ghost hit changes placement rather than the counter.\n- **Selective paths outcompete blind ones.** Once an evidence-based route out of probation existed, the random route (BIP's MRU path) became a net cost.\n\n## Recurring failures\n\n- **Stacking retention mechanisms that protect the same items** (frequency plus position in 0003, scan-skip for ghost hits in 0006): MRU placement already covers those items, so the second mechanism adds nothing.\n- **Counter-only credit or comparative frequency checks** (ghost pre-credit in 0004, the TinyLFU duel in 0004, target→probing gating in 0002): keys that never hit while resident don't benefit, and in the normal tier frequency is coupled to position, so these checks read nothing new.\n- **Anything that takes protected capacity away from ex-residents** (wider probation in 0005, set-dueling DIP in 0005, the 4× ghost window in 0006) lost hit-rate or broke even.\n- **Free-slot placement** (0001): normal buckets are almost always full, so it rarely fires.\n- **Further stash victim-choice, routing and admission-threshold tweaks** were flat in 0003. Pushing the stash threshold past roughly 3 shows diminishing returns.\n- **Long-reuse misses beyond about one cache** cannot be recovered by stash placement. Larger ghost windows let in weak evidence that displaces better residents.\n\n## Open questions\n\n- **Per-op-time and metadata-size have never been measured** after six cycles, while ghost and flag metadata keep accumulating. Cascade-free routing (0003) is still an untested per-op-time candidate.\n- **Generality:** every result comes from one LIRS trace at one cache size. Do the BIP and ghost gains, and the saturation finding, hold elsewhere?\n- **BIP endpoint:** does an even larger period help, or removing BIP entirely so the ghost is the sole exit from probation?\n- **Sharper ghost evidence:** would a window smaller than one cache, or a recency-gated window, help?\n- **Structural levers** (the stash-to-normal slot ratio, stash capacity) have not been explored.\n- **Build fidelity:** writers have not been able to inspect overlay source. The 0003 v001 result was identical to its parent, so it is still unconfirmed that changes reach the evaluated build.\n\n## Per-policy notes\n\n- **Stash (victim cache):** the main lever across all cycles. It works best as a split, with a probation region for fresh keys at the LRU end and a protected MRU region for pre-credited ex-residents and ghost-evidenced keys. Victim choice inside the stash barely matters once placement is right. Gains have been shrinking since 0004, and the stash path looks close to tuned out on this trace.\n- **Normal tier (target/probing):** admission into it was the first lever (two stash hits needed, 0001). Front landing already filters the target→probing edge. Normal-tier evictions cascade into neighbouring buckets, but cheap stash recovery made that cascade irrelevant to hit-rate.\n- **BIP/DIP insertion:** BIP fresh-key insertion is a real win, and rarer blind MRU admission is better once ghosts exist. DIP set-dueling cost hit-rate on this trace, though its signal was noisy.\n- **Ghost history:** it helps only with a window of about one cache and placement-based effects. Counter-based effects or oversized windows hurt or do nothing.\n"
}
```

### 04:50:18 thinking



### 04:50:18 chat

I've written `cross-cycle-insights.md` in the cycle 0006 cross-cycle output directory. It covers cycles 0001–0006 and has the four required sections in order. I built it only from the ledger in your prompt, without reading any earlier version of the file.

- **Recurring wins:** the cycles that gained all made fresh keys clear a high bar while protecting proven ex-residents. Where a key lands in the stash mattered more than adjusting hit counters. Space in the stash's protected region behaves as zero-sum, so keys should only get in on evidence, and ghost history only helps with a window of about one cache.
- **Recurring failures:** these were adding a second protection for items already protected, changing only the counter, taking space from the protected ex-resident region, placing items in free slots, and making ghost windows larger.
- **Open questions:** per-op-time and metadata-size have still never been measured after six cycles. Every result comes from one LIRS trace at one cache size. Other open points are where BIP should end up, sharper ghost evidence, untried structural changes such as the stash-to-normal slot ratio, and whether variant changes actually reach the evaluated build (cycle 0003's v001 scored exactly the same as its parent).
- **Per-policy notes:** grouped under the stash, the normal tier, BIP/DIP insertion and ghost history.


---

- finished: 2026-10-21T04:50:18+00:00
- duration: 28.28s
- status: ok
- input_tokens: 4
- output_tokens: 2101
- cache_read_tokens: 38500
- cache_write_tokens: 11471
- cost_usd: 0.142204
