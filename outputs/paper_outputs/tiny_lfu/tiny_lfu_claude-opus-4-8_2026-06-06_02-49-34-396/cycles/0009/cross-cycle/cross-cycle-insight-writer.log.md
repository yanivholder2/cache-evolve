# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-06-06T05:33:55+00:00

## timeline

### 05:34:13 thinking



### 05:34:13 chat

I'll write the cross-cycle insights as a fresh projection of the ledger digest.

### 05:34:23 tool: Write

```json
{
  "file_path": "cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Cost/efficiency levers that preserve the frequency signal and admission\n  logic pay reliably and nearly free: narrowed counters, a cheap\n  multiply-xorshift hasher, and compact `u32`-sentinel node links each won\n  on their target axis at zero-to-marginal hit-rate cost.\n- Composition is additive. Stacking proven winners (fast hasher + narrow\n  counters, compact layout + wider sketch) repeatedly delivered the sum of\n  the gains because the changes touch disjoint structures — confirmed\n  across several cycles.\n- Sharpening the *existing* signal beat adding new structure. Widening the\n  sketch (more columns → fewer collisions) was the strongest hit-rate\n  mover in the early-to-mid cycles, and came free until cache-locality\n  cost set in.\n- Compact integer/index encodings strictly dominate `Option`-based links:\n  smaller footprint, equal hit-rate, and slightly better tail (`!= NIL`\n  cheaper than `match`).\n\n## Recurring failures\n\n- Counter aging/decay is a settled net loss on this trace. Tried tight,\n  tried wide+amortized — it erases earned mass on a stable hot set faster\n  than it adds contrast, dropping hit-rate while taxing every increment.\n- Added admission/eviction *structure* keeps being inert: a small\n  recency window (W-TinyLFU), SLRU probation/protected split, and a\n  one-count admission margin all failed because the existing\n  `incoming_freq > victim_freq` gate already does that filtering. The\n  hit-rate ceiling is set by admission quality, not list shape.\n- Edits to the per-op hot path buy nothing once the sketch is wide:\n  conservative update (gain at factor-4 vanishes at factor-16), branchless\n  saturating increment, and removing a row all sat below the noise floor\n  or regressed per-op time.\n- Row-hash independence (real second hash vs. double-hashing) and flattening\n  the sketch allocation both resolved negatively: the correlation was never\n  the accuracy bottleneck, and the wide-sketch per-op tax is a genuine\n  footprint/cache floor, not a layout artifact.\n\n## Open questions\n\n- Is hit-rate effectively saturated at the incumbent base? Every named\n  mover — estimate rule, geometry, admission margin, eviction structure —\n  has now been ruled flat on the wide base.\n- The eviction *walk* / victim sampling is the one surface not yet mined\n  for tail/per-op savings — does it hold cost wins without touching\n  hit-rate?\n- Is there any admission-side edit that is neither a constant/margin tweak\n  nor a structural overlay, given admission is repeatedly the binding\n  constraint?\n\n## Per-policy notes\n\n- **CountMinSketch (frequency signal):** carries real slack for\n  cost-cutting (narrow counters, cheap hash) but its accuracy knobs have\n  hard limits — width saturates around factor-16, depth has a floor at 4\n  (trimming below leaks aliased estimates), and conservative update only\n  helps while collisions are common.\n- **TinyLFU admission gate:** the dominant determinant of hit-rate and\n  repeatedly the binding constraint; it already absorbs the work that\n  windowing/segmentation/margin schemes try to add, leaving them inert.\n- **Sampled-coldest eviction:** victim-sample depth is accuracy-saturated\n  (the sketch estimate, not walk length, bounds eviction quality); useful\n  only as a p99-tail/cost knob, and cheapest when it shares the single\n  cached hash fold.\n"
}
```

### 05:34:27 chat

Wrote `cross-cycle-insights.md` — a fresh projection of the 9-cycle ledger into the four fixed sections:

- **Recurring wins:** signal-preserving cost levers, additive composition, sketch-width signal sharpening, compact integer links.
- **Recurring failures:** counter aging, added admission/eviction structure (all inert against the existing gate), hot-path increment edits, and the row-independence/allocation-shape dead ends.
- **Open questions:** whether hit-rate is now saturated, the unmined eviction-walk cost surface, and non-trivial admission-side edits.
- **Per-policy notes:** grouped by sketch, admission gate, and sampled eviction.


---

- finished: 2026-06-06T05:34:27+00:00
- duration: 31.50s
- status: ok
- input_tokens: 3
- output_tokens: 2147
- cache_read_tokens: 41588
- cache_write_tokens: 11727
- cost_usd: 0.094982
