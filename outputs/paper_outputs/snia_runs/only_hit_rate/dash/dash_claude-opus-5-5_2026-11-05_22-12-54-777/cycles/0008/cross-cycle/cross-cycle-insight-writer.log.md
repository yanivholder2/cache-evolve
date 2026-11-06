# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-11-05T23:17:25+00:00

## timeline

### 23:17:48 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0008)\n\n## Recurring wins\n\n- **One isolated change per variant** gave clean attribution in every cycle. The bundled v001 in c0002 was the only cohort whose regression could not be pinned to a cause.\n- **Lineage of Winners:** 24/8 topology (c0001), then the frequency-aware 8-slot tail victim (c0002), then the ghost-credited admission ring (c0004), then a direct-mapped ghost sized to the cache (c0005), then halving on demotion into the stash (c0006), then ghost hits bypassing the stash to target Front (c0007), then a contention-gated bypass (c0008). Each step stacked cleanly on the one before.\n- **In-bucket victim choice moves the small cache size; topology does not.** The frequency filter opened small-size gains, and later levers built on it.\n- **Admission-side memory (the ghost) is the only family that reached hm_0's long-gap reuse.** Its gains came from three things: reach (c0005), where ageing happens (c0006), and above all **where credited keys land** (c0007). Changing landing was a structural fix that parameter tuning could not have found.\n- **Ageing is safe only when applied to items already judged cold**, for example on demotion into the stash. It must not penalise hits or recent Front arrivals.\n- **Protect incumbents, not newcomers** (c0008). Fast eviction of un-re-hit ghost arrivals acts as a useful filter. Gating admission on contention helped the small size.\n- **Whether a cell responds depends on whether the ghost fires.** hm_0 responds first. web_0 moved only once ghost-driven placement put returners into target at the large size.\n\n## Recurring failures\n\n- **Insert-position changes once the tail scan reads counters.** MRU promotion hurt web_0 in c0002 and again in c0003, and gated demotion did nothing. Front landing is part of how the filter works, because new arrivals must sit inside the scan window.\n- **Tuning the tail scan.** Window 8 is the peak: both narrower (c0003) and wider (c0004) lost. Cheaper scan settings could not be justified without cost data.\n- **Faster or broader ageing.** Decrement on every eviction failed in c0004 and again in c0007. Removing the stash-hit halving failed in c0005. Every time, the counters of Front-landed items drained before they could protect anything, and small caches were hit hardest.\n- **Normal/stash ratio sweeps past 3/4.** The stash is the only path into the protected tier, so shrinking it starves promotion.\n- **Ghost bonus size and ghost size above the cache.** Both were exhausted by c0006. Narrow fingerprints cost web_0 through false matches.\n- **Direct scan defences for web_0.** A TinyLFU gate on promotion did nothing, because scan keys never reach it.\n- **Protecting credited keys inside the victim scan** (c0008). This moves eviction onto proven residents and hurt every cell.\n- **Changing where cascade victims go** (stash vs ghost). Hit-rate was identical either way.\n- **Measurement gap:** per-op-time and metadata-size have gone unmeasured for 8 cycles. Cost-reduction variants cannot be judged, and the eviction and admission paths keep growing.\n\n## Open questions\n\n- Can a relative bypass gate win back the large-size web_0 gain lost in c0008 while keeping the small-size gains? One option is to bypass when the returner's credit is at least the window's minimum counter.\n- Should bypass/stash rules depend on cache size or bucket type, since the trade-off now changes with capacity?\n- On a refused bypass, does keeping the ghost entry so the key can retry on a second return help, or does it only delay the loss?\n- Is large-size hm_0 structurally flat now? Do its lost keys leave through paths the ghost never records?\n- Are there other paths that still send proven-reuse keys through the stash, such as demoted items with prior target hits?\n- What do the current Winner's ghost lookup, bypass cascade and 8-slot scan actually cost in per-op-time and metadata-size?\n- Gains are shrinking. Is a structurally different hypothesis needed for web_0 at the small size?\n\n## Per-policy notes\n\nThese notes cover the single evolving lineage of the segmented normal/stash policy with a frequency-aware tail and a ghost.\n\n- **Topology:** a 24/8 split (about 3/4 normal) is settled. Do not revisit it.\n- **Tail victim scan:** keep the 8-slot window, Front landing for arrivals and promotions, and slow halve-on-cap ageing. This setup is load-bearing and its parameters are fully explored.\n- **Stash:** halving on a stash hit is the stash's only ageing and must stay. Halving on demotion into the stash is beneficial. If stash ageing changes, move it rather than remove it.\n- **Ghost:** direct-mapped, sized to the cache, full-width fingerprints, credit 1. Hits go straight to target Front, gated on whether a cold slot is free.\n- **hm_0:** the most responsive trace in both directions. The small-size cell is the most sensitive and shows gains first.\n- **web_0:** largely inert. It reacts only through ghost-driven placement at the large size, and it is hurt by false matches and over-crediting.\n- **Process:** variant insights should confirm that the overlay diff landed as specified. In c0008 two variants' mechanisms were only inferred.\n"
}
```

### 23:17:54 thinking



### 23:17:54 chat

I wrote `cross-cycle-insights.md` from the ledger for cycles 0001–0008, using the four required sections in order.

- **Recurring wins:** Every Winner built cleanly on the one before. The chain runs from the 24/8 topology through the frequency-aware tail victim and the ghost improvements to the contention-gated stash bypass. The biggest step came from changing where ghost-credited keys land, not from tuning parameters. The other wins:
  - Ageing works only when it hits items already judged cold.
  - Protecting existing residents helps more than protecting newcomers.
  - A trace responds only when the ghost fires on it.
- **Recurring failures:**
  - Changing insert positions once the tail scan reads counters.
  - Tuning the scan window.
  - Faster or broader ageing.
  - Shrinking the stash below about a quarter of the cache.
  - More ghost bonus or a ghost larger than the cache.
  - Defences aimed directly at scans.
  - Protecting credited keys inside the victim scan.
  - Changing where cascade victims go.

  Per-op-time and metadata-size have now gone unmeasured for 8 cycles.
- **Open questions:**
  - Can a relative bypass gate win back the large-size web_0 gain lost in cycle 0008?
  - Should bypass rules depend on cache size?
  - Does letting a refused key retry on its next return actually help?
  - Is large-size hm_0 flat for structural reasons?
  - What do the current mechanisms actually cost?
  - Do the shrinking gains call for a structurally different idea?
- **Per-policy notes:** These cover the one evolving policy: which settings are settled for the topology, tail scan, stash ageing and ghost, how hm_0 and web_0 each tend to respond, and a reminder that variant write-ups should confirm their change landed as specified.


---

- finished: 2026-11-05T23:17:55+00:00
- duration: 30.16s
- status: ok
- input_tokens: 4
- output_tokens: 2612
- cache_read_tokens: 42367
- cache_write_tokens: 14383
- cost_usd: 0.175793
