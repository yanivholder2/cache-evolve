# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-21T05:03:29+00:00

## timeline

### 05:03:46 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Filter admission instead of adding protection.** From 0001 to 0008, nearly every winner made it harder to get into scarce space: first the normal tier (stash-hit threshold), then protected stash space (BIP, rarer BIP, no BIP, recency-gated ghost). Variants that widened access or added protection lost or came out flat.\n- **Fresh keys face a harsh bar; proven ex-residents are protected.** This SLRU/2Q-style split ran through every winner from 0002 on: MRU stash re-entry for demoted items, LRU-end (probation) insertion for fresh keys, and MRU placement only when there is evidence of reuse.\n- **Placement beats counters.** Moving a key changed hit-rate; crediting its counter did not. Ghosts failed as counter pre-credit (0004) and won once a ghost hit changed placement (0005).\n- **Recent evidence beats more evidence.** A ghost window of about one cache helped, a 4× window hurt, and a recency gate at about half the window helped. The useful reuse horizon is shorter than one cache's worth of evictions.\n- **When one selective admission path exists, cut the blind ones.** Making BIP rarer, then removing it, paid off each time after the ghost existed. Removal stacked with the recency gate, and it also simplifies the design.\n\n## Recurring failures\n\n- **Taking capacity from the protected ex-resident region.** Wider probation, set-dueling DIP and over-wide ghost windows all lost. Protected stash capacity is close to zero-sum.\n- **Stacking retention mechanisms that protect the same items.** The CLOCK scan on top of MRU placement, scan-skip for ghost hits, and a threshold of 3 on top of MRU re-entry added nothing.\n- **Counter or frequency gates where position already decides.** The frequency bar on target→probing, the TinyLFU-style duel, and ghost counter pre-credit were all flat, because Front landing couples frequency to position.\n- **Free-slot placement and normal-tier MRU landing.** Normal buckets are almost always full, and MRU landing lowers the bar for entering probing.\n- **Micro-tuning in-stash parameters.** Threshold, victim choice, gate cutoff and graded mid-stash placement now each move hit-rate by about noise.\n- **Per-op-time hypotheses.** Cascade-free routing was tried twice and showed no hit-rate signal, once bit-identical to its parent. Per-op-time has no fitness dimension, so these hypotheses cannot be scored.\n- **Process failure.** The continuations asked for more evaluation coverage every cycle, and it never happened. Every result comes from one LIRS trace at one cache size, scored on hit-rate only, and writers cannot inspect overlay diffs.\n\n## Open questions\n\n- Do the gains (BIP and its removal, gated ghost) hold on other traces and cache sizes, especially workloads with useful reuse beyond the ghost horizon?\n- Can the ghost table shrink to the gated horizon without losing hit-rate? This is the main metadata lever.\n- Do metadata-size and per-op-time (ghost table, flags) justify the small recent gains? Neither has ever been measured.\n- Does the cascade/displacement path fire at all at this size? Did some null variants (0003 v001, 0007 v003) actually reach the evaluated build?\n- Are structural levers (the stash-to-normal slot ratio, or who reaches the stash at all) the remaining source of hit-rate now that in-stash levers look exhausted?\n\n## Per-policy notes\n\n- **Stash→target admission:** a threshold of 2 to 3 is near optimal. The edge is cheap to filter because the stash is larger than the normal tier.\n- **Target→probing edge:** admission gating does nothing here, because Front landing already filters it. Leave it alone.\n- **Normal tier:** the bucket is the unit that fills, and evictions cascade into neighbours. That damage stopped mattering for hit-rate once the stash recovers demoted items cheaply.\n- **Stash (victim cache):** MRU re-entry for ex-residents, LRU-end insertion for fresh keys, MRU only on a recent ghost hit, no BIP. The insert-position knobs are tuned out on this trace.\n- **Ghost history:** it works only when a hit changes placement, the window is at most about one cache, and the hit is recency-gated. Graded placement and tighter cutoffs are on a plateau.\n- **BIP/DIP:** it was useful as a bridge before the ghost existed, and its optimum is now \"off\". Adaptive DIP lost, but its duel signal was noisy.\n"
}
```

### 05:03:52 chat

I wrote `cross-cycle-insights.md` from the ledger alone, covering cycles 0001 to 0008 in the four required sections.

- **Main finding:** in almost every cycle, the winner made it harder to get into scarce space. Variants that added protection or widened access lost or came out flat. Placing items by evidence beat crediting their counters, and recent ghost hits beat ghost hits from further back.
- **Approaches that keep failing:**
  - Taking capacity away from the protected stash region.
  - Stacking retention mechanisms that protect the same items.
  - Counter or frequency gates where bucket position already decides.
  - Free-slot placement.
  - Further in-stash micro-tuning, which now moves hit-rate by only about noise.
- **A recurring process gap:** every cycle's continuation asked for more evaluation coverage, and it never happened. All eight cycles used one LIRS trace at one cache size and scored hit-rate only. Per-op-time and metadata-size have never been measured. Because writers can't see the overlay diffs, two null results (0003 v001 and 0007 v003) may be changes that never reached the evaluated build.
- **Open questions:**
  - Do the gains hold on other traces and cache sizes?
  - Can the ghost table shrink to the gated horizon without losing hit-rate?
  - Does the ghost table's metadata and per-op cost justify its gains?
  - Are structural changes, such as the stash-to-normal slot ratio or who reaches the stash at all, the remaining levers?

The per-policy notes summarise where each part of the design has landed: the two promotion edges, the normal tier, the stash, the ghost history and BIP/DIP.


---

- finished: 2026-10-21T05:03:52+00:00
- duration: 22.56s
- status: ok
- input_tokens: 4
- output_tokens: 2257
- cache_read_tokens: 40396
- cache_write_tokens: 13420
- cost_usd: 0.161485
