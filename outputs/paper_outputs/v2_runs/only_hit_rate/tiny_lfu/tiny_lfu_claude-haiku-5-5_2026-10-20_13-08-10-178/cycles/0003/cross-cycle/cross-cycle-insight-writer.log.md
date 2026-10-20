# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-20T13:26:55+00:00

## timeline

### 13:27:02 thinking



### 13:27:06 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Layout-only changes to the frequency sketch have not regressed hit-rate in any cycle so far (Cycles 0001–0003). They are safe to keep, but no hit-rate gain has been shown from them.\n- Restructuring recency order inside the main region (probation/protected split) is the only change so far to move hit-rate upward (Cycle 0003). It leaves the sketch and admission path untouched.\n\n## Recurring failures\n\n- Changes that alter sketch behaviour lose hit-rate: fixed-schedule halving (Cycle 0001) and a changed key-to-counter mapping (Cycle 0001, small dip). Decay discards admission signal at this capacity rather than removing stale history.\n- Changes that move when or where admission sees a new key lose hit-rate: a recency window ahead of the LRU with admission at window exit (Cycle 0002). The one-touch-burst benefit it targeted did not show up on this trace.\n- Claimed gains from layout work (memory footprint, per-op time) keep landing on dimensions the fitness vector does not measure, so those Winners advance only by not regressing.\n\n## Open questions\n\n- Do layout-only Winners gain anything once a memory or timing dimension is in the fitness vector, and do they justify keeping them?\n- Does the probation/protected gain hold on other traces and cache sizes, or is it specific to the current trace?\n- Does the sketch's saturation point bind on very hot keys, creating admission ties?\n- Would a gentler or staleness-gated decay recover signal without the admission losses seen so far?\n- Is the small hit-rate dip from the key-to-counter mapping change within noise?\n\n## Per-policy notes\n\n- TinyLFU-style admission (frequency sketch plus admission comparison): every sketch-side or admission-timing change so far has cost hit-rate. Keep sketch counts and the admission rule fixed; treat layout work here as hit-rate-neutral only.\n- Window-in-front-of-LRU: tried once (Cycle 0002), lost hit-rate. Do not re-propose at this capacity.\n- Main-region LRU (segmented into probation and protected): the only hit-rate gain so far (Cycle 0003). Provisional until tested beyond a single trace and size; build on it before adding other changes.\n- Across the ledger, all cycles have used one eviction-policy family and one trace at one cache size, so policy-level conclusions are narrow.\n"
}
```

### 13:27:09 chat

I wrote `cross-cycle-insights.md` to the cross-cycle output directory, using the four required sections in order. I built it only from the inlined ledger digest and did not read any earlier version of the file.

The main points:

- **Recurring wins:** Layout-only sketch changes have not lowered hit-rate in any of the three cycles, but none has raised it either. The only hit-rate gain so far is the probation/protected split of the main region from Cycle 0003, which is provisional because it rests on one trace and one cache size.
- **Recurring failures:** Changes that alter sketch behaviour (fixed-schedule halving, the key-to-counter mapping) and changes that alter when admission sees a key (the recency window ahead of the LRU) have all cost hit-rate. Layout work keeps claiming gains on dimensions the fitness vector does not measure.
- **Open questions:** Whether layout gains show up once a memory or timing dimension is added, whether the main-region gain generalises, whether the sketch saturates on hot keys, whether gentler decay helps, and whether the key-to-counter dip is noise.
- **Per-policy notes:** Grouped by TinyLFU-style admission, the window-in-front-of-LRU, and main-region LRU. The ledger covers one eviction-policy family, one trace and one cache size, so the policy-level conclusions are narrow.


---

- finished: 2026-10-20T13:27:10+00:00
- duration: 14.78s
- status: ok
- input_tokens: 4
- output_tokens: 2350
- cache_read_tokens: 35623
- cache_write_tokens: 8070
- cost_usd: 0.003146
