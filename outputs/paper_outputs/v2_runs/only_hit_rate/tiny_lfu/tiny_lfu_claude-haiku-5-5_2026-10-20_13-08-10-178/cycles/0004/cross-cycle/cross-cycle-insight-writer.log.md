# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-20T13:31:03+00:00

## timeline

### 13:31:10 thinking



### 13:31:14 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changes that keep the sketch's counts and the admission rule intact, and reorder recency inside the main region, are the lever that has produced a hit-rate gain (probation/protected split, Cycle 0003).\n- Widening the frequency sketch to cut over-counting from colliding keys is the one sketch-side change that has moved hit-rate up; sketch changes aimed at collision error look more promising than those aimed at decay or key mapping (Cycle 0004).\n- Layout-only sketch changes reliably hold hit-rate flat, so they can advance as non-regressions, but they have not shown a gain on the measured fitness dimension.\n\n## Recurring failures\n\n- Changing when or where admission sees a new key (e.g. a recency window in front of the LRU) costs hit-rate on this trace (Cycle 0002).\n- Fixed-schedule halving of the sketch discards history that admission still needs at this capacity (Cycle 0001).\n- Key-to-counter remapping produced a small hit-rate dip and has not been shown to be noise (Cycle 0001).\n- Main-region variants that shrink the protected segment or push demoted protected entries to the front of probation both lost hit-rate; the current ratio and back-of-probation demotion are the incumbent settings (Cycle 0004).\n- Layout Winners justify themselves on footprint or per-op cost, which the fitness vector does not measure, so their payoff stays unconfirmed (Cycles 0001–0003).\n\n## Open questions\n\n- Do the probation/protected and sketch-width gains hold on other workloads and cache sizes? Every result so far comes from one trace at one size.\n- What metadata, memory, or timing signal would confirm the layout gains, and is the sketch-width gain worth its metadata cost once measured?\n- At what width does further sketch widening stop helping, and does counter saturation ever bind on very hot keys?\n- Is the key-remapping dip within noise, or a real regression?\n- Would a gentler or staleness-gated sketch decay recover signal without the admission losses seen so far?\n- Is there an admission structure that helps one-touch bursts without giving up sketch signal?\n\n## Per-policy notes\n\n- TinyLFU frequency sketch (admission estimate): width increases help; decay on a fixed schedule hurts at this capacity; layout-only storage changes are neutral on hit-rate but unmeasured on cost.\n- TinyLFU admission path: changes that alter when or where a new key is considered for admission have lost hit-rate; keep the admission comparison fixed until other changes are settled.\n- Main region (SLRU-style probation/protected): the probation/protected split is the best-performing structure so far; the current protected ratio and back-of-probation demotion are the settings to hold, and the split needs validation beyond one trace.\n"
}
```

### 13:31:17 chat

I wrote `cross-cycle-insights.md` to the cycle 0004 cross-cycle output directory. It is a fresh projection of the four-cycle ledger digest, with the four required sections in order. I did not read the previous version of the file.

The main signals it records:
- **Wins:** Reordering recency inside the main region (the probation/protected split, from cycle 0003) and widening the frequency sketch to reduce collision over-counting (cycle 0004) are the only changes that have moved hit-rate up. Layout-only sketch changes have held hit-rate flat but have not shown a measured gain.
- **Failures:** Changing when or where admission sees a new key, fixed-schedule sketch halving, and shrinking the protected segment or pushing demoted entries to the front of probation have all cost hit-rate. Layout Winners' claimed benefits sit on dimensions the fitness vector doesn't measure.
- **Open questions:** Every result comes from one trace at one cache size, so none of the gains are established yet. The file also asks where sketch widening stops helping and whether the small key-remapping dip is noise.

The per-policy section is grouped by the TinyLFU frequency sketch, the admission path, and the main region's probation/protected structure. It contains no numbers beyond the cycle references.


---

- finished: 2026-10-20T13:31:18+00:00
- duration: 14.83s
- status: ok
- input_tokens: 4
- output_tokens: 2515
- cache_read_tokens: 36271
- cache_write_tokens: 8910
- cost_usd: 0.003403
