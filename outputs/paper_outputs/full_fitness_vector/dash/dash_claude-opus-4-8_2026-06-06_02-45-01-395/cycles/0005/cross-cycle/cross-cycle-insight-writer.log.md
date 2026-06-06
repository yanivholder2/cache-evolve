# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-06-06T05:53:40+00:00

## timeline

### 05:53:51 thinking



### 05:53:58 tool: Write

```json
{
  "file_path": "cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Removing hidden hot-path cost reliably trims latency: O(n) front-shifts,\n  redundant candidate scans, and per-op syscalls (`Instant::now`) were all\n  cheap, repeatable wins early on.\n- Per-entry metadata slimming paid off until exhausted: under a positional\n  policy the `Item` carried timestamp + frequency fields the policy never\n  read, so deleting them cut size and latency for free down to the key+value\n  floor.\n- The structure-vs-layout pivot beat per-entry tuning once layout saturated:\n  relocating the promotion-heat signal into a segment-side table reached the\n  long-sought Pareto corner (best hit-rate AT the metadata floor).\n- Placement and footprint are orthogonal — porting a slim `Item` onto the\n  MRU-promotion winner stacked both wins with no hit-rate cost.\n- Devirtualizing the inner eviction-policy trait object produced a\n  harness-resolvable latency win, establishing the granularity of change\n  that is actually measurable.\n\n## Recurring failures\n\n- Tuning the promotion gate (width, positional vs frequency shape) keeps not\n  paying off: MRU placement already captures the warm mass, so gate shape and\n  placement are substitutes, not additive — gains land in the noise floor.\n- Chasing hit-rate on the hot `get` path costs real latency (promotion\n  cascades, per-hit counter RMW) for indistinguishable gain.\n- Micro-latency reorders (single scan-per-hit savings) sit below measurement\n  resolution and often move the wrong way due to variance.\n- Raising set-associativity left hit-rate bit-for-bit unchanged — misses are\n  capacity/compulsory-bound, not conflict-bound.\n- Frequency/admission (ghost-list) levers moved hit-rate nothing, only adding\n  put-side bookkeeping cost.\n- Stacking the size (side-table) and latency (devirtualization) winners on\n  one hot path is substitutive: the side-table hash probe, not the vtable,\n  bounds latency, so the two speedups cannot be collected together.\n- Outer-layer monomorphization regressed everything: replacing a compact heap\n  pointer with a fat inline struct widened `Vec` stride and cost locality more\n  than the saved dispatch.\n\n## Open questions\n\n- Is there a hot-path layout where side-table relocation and devirtualization\n  stop competing, or are the metadata-floor corner and latency floor mutually\n  exclusive?\n- Is there a structural change that removes indirection while preserving or\n  improving locality (rather than fattening layout or adding bookkeeping), or\n  has latency also saturated?\n- Where does the next size win come from now that per-entry layout is at its\n  floor — bucket/segment structure, allocation strategy?\n- Can any latency win reduce p99 rather than just p50? Side tables so far\n  trade tail latency for size.\n\n## Per-policy notes\n\n- ClassicLRU (active eval policy): purely positional — ignores recency/\n  frequency metadata, which is why metadata trimming was near-free and why\n  frequency/admission levers do nothing.\n- Hit-rate under ClassicLRU is confirmed capacity/compulsory-capped across\n  every tried class: placement, gate shape, associativity, and admission all\n  exhausted. Treat hit-rate as fully closed.\n- The frontier stands as two banked, substitutive corners: a devirtualized\n  latency baseline and a side-table size corner at the metadata floor.\n"
}
```

### 05:54:01 chat

Wrote `cross-cycle-insights.md` — a fresh projection of the 5-cycle ledger across the four fixed sections. Key threads carried forward: latency wins come from removing hidden hot-path cost (not micro-reorders); hit-rate is fully capacity/compulsory-capped under the positional ClassicLRU; and the frontier is two substitutive corners (devirtualized latency baseline + side-table size corner) that resist stacking on a shared hot path.


---

- finished: 2026-06-06T05:54:01+00:00
- duration: 21.35s
- status: ok
- input_tokens: 3
- output_tokens: 1445
- cache_read_tokens: 38168
- cache_write_tokens: 7684
- cost_usd: 0.065929
