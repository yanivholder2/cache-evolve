# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:21:38+00:00

## timeline

### 04:21:52 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **Controlling who gets into the scarce normal tier matters more than where items are placed.** Stricter stash→target admission won in cycle 1 and still gained something at a higher threshold in cycle 2. It costs little because the stash is larger than the normal tier, so unproven keys can wait there.\n- **Treat fresh keys and proven ex-residents differently (SLRU/2Q-like).** Fresh keys face a high bar to enter. Items demoted from the normal tier get protection (MRU stash re-entry) and a pre-credit so they can come back cheaply. Every variant that gained in cycle 2 used this split.\n- **Retention changes and admission changes complement each other.** The pre-credit only pays off if the demoted item stays in the stash long enough to get its return hit. That depends on both where it is placed and how big the credit is relative to the threshold.\n- **Build each cycle on the previous winner.** The lineage is cycle-1 v003 (two stash hits before promotion), then cycle-2 v003 (MRU re-entry for demoted items), and each step improved hit-rate.\n\n## Recurring failures\n\n- **Changes that loosen the bar for moving up hurt hit-rate.** MRU landing in the target bucket (cycle 1) also lowered the bar for promotion to probing, because the \"is hottest\" check reads bucket position. That cost hit-rate.\n- **\"Free slot\" placement ideas barely work.** In steady state the normal buckets are almost always full together, so spilling into a free probing slot rarely happens.\n- **A frequency gate on the target→probing edge did nothing.** Front landing already filters that edge, and its hit counter accumulates and is pre-credited to foreign residents, so the gate filters almost nothing. Churn from this edge seems small anyway.\n- **Pushing one admission threshold higher and higher gives less each time.** The stash threshold looks close to its best value at 3. Don't go higher.\n- **More normal-tier moves cause collateral damage.** Each normal-tier eviction starts a demotion cascade into a neighbouring bucket, so changes that add moves hurt other buckets as well.\n\n## Open questions\n\n- Do MRU stash re-entry and the threshold of 3 (with threshold-relative pre-credit) stack when combined? Or do protected ex-residents crowd out one-touch keys that are waiting for their third hit?\n- Would routing evictions differently (where demoted or cascaded items go) beat any further admission tuning?\n- Other stash retention controls: the depth at which demoted items land, and explicit protection for pre-credited items.\n- Could MRU landing in the target bucket work if a separate hit count kept the bar for entering probing where it is?\n- How well does this generalize? Every result so far is hit-rate only, on one LIRS trace at one cache size. Per-op-time and metadata-size have never been measured for any winner. Scan-heavy and larger-working-set behaviour, especially for MRU re-entry, is untested.\n\n## Per-policy notes\n\n- **Tiered stash / target / probing bucketed policy (the only lineage so far):**\n  - The normal tier is the bottleneck. The stash works best as a large holding area plus a filter, and it has started to act as a victim cache for demoted items.\n  - Item position and promotion thresholds are coupled through the \"is hottest\" check, so any change to placement also changes the effective promotion thresholds.\n  - Most of the remaining hit-rate gains on this trace seem to be in stash retention and in eviction/cascade routing, not in the target→probing edge.\n"
}
```

### 04:21:58 chat

I wrote `cross-cycle-insights.md` in the cycle 0002 cross-cycle folder. It has the four sections in the required order and is built only from the two ledger rows. It contains no numbers except the thresholds the ledger names (2 and 3 stash hits).

- **Recurring wins:** Stricter admission to the normal tier keeps paying off. On top of that, cycle 2 added protection for items demoted out of the normal tier: they re-enter the stash at the MRU end and come back cheaply. The two work together, and each cycle improved by building on the previous winner.
- **Recurring failures:** Placing promoted items at MRU in their bucket lowered the bar for moving up and lost hit-rate. Spilling into a free slot rarely fires because the buckets are almost always full. A frequency gate on the target→probing edge did nothing. Raising the stash threshold past 3 looks like diminishing returns.
- **Open questions:**
  - Do MRU re-entry and the threshold of 3 stack when combined?
  - Would routing evictions to different buckets beat more admission tuning?
  - What other stash retention controls are worth trying?
  - Every result so far is hit-rate only, on one LIRS trace at one cache size. Per-op-time and metadata-size have never been measured for any winner.
- **Per-policy notes:** Everything so far comes from one tiered stash/target/probing policy. The normal tier is the bottleneck, and where an item sits in its bucket is tied to how many hits it needs to move up. Most of the remaining gains look to be in stash retention and eviction routing.


---

- finished: 2026-10-21T04:21:58+00:00
- duration: 20.20s
- status: ok
- input_tokens: 4
- output_tokens: 1849
- cache_read_tokens: 34553
- cache_write_tokens: 7205
- cost_usd: 0.101851
