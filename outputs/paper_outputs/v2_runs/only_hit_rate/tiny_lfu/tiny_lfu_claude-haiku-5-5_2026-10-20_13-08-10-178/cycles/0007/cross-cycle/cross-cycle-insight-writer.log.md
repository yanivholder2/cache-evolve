# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-20T13:43:22+00:00

## timeline

### 13:43:28 thinking



### 13:43:33 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-cycle insights\n\n## Recurring wins\n\n- Reducing collision over-counting in the frequency sketch is the lever that has most consistently produced hit-rate gains: sketch widening (0004), conservative update (0005), and an extra hash row (0007) all acted on the same estimator error. Gains appear to overlap rather than stack independently.\n- Keeping the sketch's min-across-rows estimate well supplied with independent rows matters; removing depth hurt (0005) and adding it helped (0007).\n- The probation/protected split inside the main region is a real lever (0003, 0006), but its effect is non-monotone: the incumbent ratio sits near the best setting, and pushing the protected share higher has reversed a gain (0007).\n- Promoting entries on every hit has held up better than gating promotion on sketch estimates (0007).\n\n## Recurring failures\n\n- Decaying or aging sketch history on a fixed schedule or access-count trigger has consistently lost hit-rate (0001, 0005): admission needs accumulated frequency at these capacities.\n- Changing when or whether admission sees a key has lost hit-rate: a recency window ahead of the LRU (0002), a doorkeeper that withholds first-touch keys from the sketch (0006), and sketch-gated promotion (0007). Frequency counts lag recency, and correction at estimate time does not replace withheld increments.\n- Rewrites of key-to-counter mapping and sketch-side changes that shift estimates the wrong way did not pay off (0001).\n- Layout-only changes (narrower counters, flattened or hoisted indices) preserve hit-rate by construction, but their claimed gains (footprint, per-op cost) sit on dimensions the fitness vector does not measure (0001, 0002, 0003). They have advanced only by not regressing, and should not be stacked until a metadata or timing signal exists.\n- Halved protected caps and push-to-front demotion lost hit-rate against the incumbent (0004).\n\n## Open questions\n\n- Winner attribution is unresolved: cycle 0007 lists v001 as a Winner but the only reported gain is v002's depth change. Confirm which variant actually advanced before building on either.\n- Do the estimator gains (width, conservative update, depth) hold on other workloads and cache sizes, and do they overlap or stack?\n- Would a memory or timing dimension in the fitness vector confirm the layout gains and price the counter, row, and per-op costs of the estimator changes?\n- Where does adding sketch rows or widening stop paying for its memory, and where does the protected-share optimum sit on other traces?\n- Is there an aging trigger, not tied to access count, that sheds stale frequency without weakening admission?\n\n## Per-policy notes\n\n- TinyLFU admission and frequency sketch: estimator error (width, conservative update, depth) is the productive direction. Decay, aging, and first-touch gating have all lost hit-rate. Keep the sketch's history intact.\n- Segmented LRU (main region): the probation/protected split helps; keep the protected share near the incumbent ratio, and keep demotion placement at the back of probation.\n- Promotion rule: promote on every hit; do not gate promotion on sketch estimates.\n- Window / LRU front region: a recency window ahead of the main LRU with admission at window exit lost hit-rate on this trace; drop this structure at current capacity.\n- All observations come from one trace at one cache size and should be treated as provisional.\n"
}
```

### 13:43:36 chat

I wrote `cross-cycle-insights.md` to the cycle 0007 cross-cycle output directory, with the four required sections in order. I built it only from the inlined ledger digest and did not read the previous file.

The main points:

- **Recurring wins:** Reducing over-counting in the frequency sketch has produced hit-rate gains in three cycles (widening in 0004, conservative update in 0005, an extra hash row in 0007). The protected-share lever helped in 0003 and 0006, but 0007 showed it peaks near the incumbent ratio.
- **Recurring failures:** Fixed-schedule or access-count decay of the sketch, changes that withhold keys from admission, and sketch-gated promotion have all lost hit-rate. Layout-only changes preserve hit-rate but their claimed gains are on dimensions the fitness vector does not measure.
- **Open questions:** The cycle 0007 Winner designation is inconsistent: the list names v001, but the only reported gain is v002's depth change. This should be resolved before anything is built on either. The other open threads are whether the estimator gains hold on other traces and sizes, and whether a memory or timing signal is needed.
- **Per-policy notes:** Grouped under TinyLFU admission, the segmented LRU main region, promotion rule, and the window/LRU front region.

Every result comes from one trace at one cache size, so the file marks those findings as provisional.


---

- finished: 2026-10-20T13:43:37+00:00
- duration: 14.96s
- status: ok
- input_tokens: 4
- output_tokens: 2466
- cache_read_tokens: 38482
- cache_write_tokens: 11027
- cost_usd: 0.003824
