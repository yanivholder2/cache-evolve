# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-06-06T05:46:25+00:00

## timeline

### 05:46:42 thinking



### 05:46:56 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changes that *remove* hot-path work pay for themselves; changes that *add* a branch or store rarely do. The biggest movers — the fast integer hasher, the metadata fold to a 24-byte node, gentle ghost admission, and flat CLOCK dropping the recency list — each cut work and so came free or near-free on multiple axes.\n- The size/representation *mechanism* is reliably cheap: spare-high-bit flags, `u32` sentinel links, and packing the segment flag into a link consistently hold the node-byte target with little or no latency tax, especially with centralized cast helpers. When these variants lost, the loss was always the encoded *policy*, never the encoding.\n- Orthogonal levers stack: the policy axis (segmentation/CLOCK) and the per-op-time axis (cheap hasher, surgery skips) have repeatedly improved together with neither paying for the other.\n- Small geometry nudges can buy hit-rate for free even though large ones don't — effects here are magnitude-dependent, not categorical.\n- A genuinely structural change can reach room that micro-edits never could (flat CLOCK beat the long-standing SLRU champion on latency and footprint at equal hit-rate after many \"at the frontier\" cycles).\n\n## Recurring failures\n\n- Admission cleverness keeps losing on this short-reuse trace: ghost/reuse-distance re-admission and two-hit probation gates either fire too rarely to move recall or drain genuine repeaters before their second hit, dropping hit-rate while adding bookkeeping. Treat as tried-and-regressed.\n- Latency micro-opts aimed at non-bottlenecks consistently regress via codegen damage (register/code-size pressure, unsafe risk): bounds-check elision, relink/store fusion, and probe-skipping get caches all attacked work that was already free. The real hot-path cost is hashing, link surgery, and cache-line traversal.\n- Skipping the HashMap probe never pays — the zero-sized xor-multiply hasher plus arena index is cheaper than any guard layered in front of it.\n- Bare data-dependent branches on the universal hot path mispredict and regress; position-gated skips pay only where the predicted position is actually common (i.e. inside the SLRU protected MRU, not on flat LRU).\n- Large geometry jumps (90/10, 15/16) starve the fast path; confirmed cliff above 7/8.\n\n## Open questions\n\n- Does the SLRU protected/probation split add anything over flat CLOCK on this trace, or was the whole segmented lineage carrying cost without payoff here?\n- Is CLOCK's slight edge over exact LRU real signal for short-reuse, or noise?\n- Where is remaining flat-CLOCK headroom — hand placement, sweep bound, fresh-key admission — now that the segmented levers are superseded?\n- Does growing the cache struct footprint carry a real latency tax worth treating as a constraint, or is the footprint-to-p50 coupling non-causal (the `u32`-narrowing control refuted it)?\n\n## Per-policy notes\n\n- SLRU: long-reigning champion across mid cycles; hit-rate has two levers — retention (geometry) and admission. Its protected-MRU fast path is already write-free, so adding any per-hit store (CLOCK bit, ghost bookkeeping) backfires there.\n- CLOCK: wins only on a flat single list, where there is no write-free MRU branch to lose; one reference bit matches LRU hit-rate while removing the recency list, cutting latency and footprint together.\n- Plain single-list LRU: surgery-skip fast path does not generalize here — the touched node is rarely the tail, so the position guard mispredicts.\n- Ghost / reuse-distance admission: unconditional promote-to-protected overshoots and evicts hot keys; gentler gating helped once (Cycle 0004) but did not reproduce on the short-reuse workload — trace/size-contingent, now treated as refuted.\n"
}
```

### 05:47:06 thinking



### 05:47:12 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changes that *remove* hot-path work pay for themselves; changes that *add* a branch or store rarely do. The biggest movers — the fast integer hasher, the metadata fold to a 24-byte node, gentle ghost admission, and flat CLOCK dropping the recency list — each cut work and so came free or near-free on multiple axes.\n- The size/representation *mechanism* is reliably cheap: spare-high-bit flags, `u32` sentinel links, and packing the segment flag into a link consistently hold the node-byte target with little or no latency tax, especially with centralized cast helpers. When these variants lost, the loss was always the encoded *policy*, never the encoding.\n- Orthogonal levers stack: the policy axis (segmentation/CLOCK) and the per-op-time axis (cheap hasher, surgery skips) have repeatedly improved together with neither paying for the other.\n- Small geometry nudges can buy hit-rate for free even though large ones don't — effects here are magnitude-dependent, not categorical.\n- A genuinely structural change can reach room that micro-edits never could (flat CLOCK beat the long-standing SLRU champion on latency and footprint at equal hit-rate after several \"at the frontier\" cycles).\n\n## Recurring failures\n\n- Admission cleverness keeps losing on this short-reuse trace: ghost/reuse-distance re-admission and two-hit probation gates either fire too rarely to move recall or drain genuine repeaters before their second hit, dropping hit-rate while adding bookkeeping. Treat as tried-and-regressed.\n- Latency micro-opts aimed at non-bottlenecks consistently regress via codegen damage (register/code-size pressure, unsafe risk): bounds-check elision, relink/store fusion, and probe-skipping get caches all attacked work that was already free. The real hot-path cost is hashing, link surgery, and cache-line traversal.\n- Skipping the HashMap probe never pays — the zero-sized xor-multiply hasher plus arena index is cheaper than any guard layered in front of it.\n- Bare data-dependent branches on the universal hot path mispredict and regress; position-gated skips pay only where the predicted position is actually common (inside the SLRU protected MRU, not on flat LRU).\n- Large geometry jumps (90/10, 15/16) starve the fast path; a sharp cliff sits just above the 7/8 protected cap.\n\n## Open questions\n\n- Does the SLRU protected/probation split add anything over flat CLOCK on this trace, or was the whole segmented lineage carrying cost without payoff here?\n- Is CLOCK's slight edge over exact LRU real signal for short-reuse, or noise?\n- Where is remaining flat-CLOCK headroom — hand placement, sweep bound, fresh-key admission — now that the segmented levers are superseded?\n- Does growing the cache struct footprint carry a real latency tax worth treating as a constraint, or is the footprint-to-p50 coupling non-causal (the `u32`-narrowing control refuted it)?\n\n## Per-policy notes\n\n- SLRU: long-reigning champion across the middle cycles; hit-rate has two levers — retention (geometry) and admission. Its protected-MRU fast path is already write-free, so adding any per-hit store (CLOCK bit, ghost bookkeeping) backfires there.\n- CLOCK: wins only on a flat single list, where there is no write-free MRU branch to lose; one reference bit matches LRU hit-rate while removing the recency list, cutting latency and footprint together.\n- Plain single-list LRU: the surgery-skip fast path does not generalize here — the touched node is rarely the tail, so the position guard mispredicts.\n- Ghost / reuse-distance admission: unconditional promote-to-protected overshoots and evicts hot keys; gentler gating helped once (Cycle 0004) but did not reproduce on the short-reuse workload — trace/size-contingent, now effectively refuted.\n"
}
```

### 05:47:18 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the 9-cycle ledger, using the four fixed sections.

Key cross-cycle signals captured:
- **Recurring wins**: work-*removing* changes pay off (fast hasher, 24-byte fold, flat CLOCK); the size/sentinel *mechanism* is reliably cheap while only the encoded *policy* loses; orthogonal axes stack; small geometry nudges are free where large ones aren't; structural changes reach room micro-edits can't.
- **Recurring failures**: admission cleverness on the short-reuse trace; latency micro-opts against non-bottlenecks; probe-skipping; mispredicting bare hot-path branches; oversized geometry jumps.
- **Open questions**: whether segmentation still earns its keep over flat CLOCK, whether CLOCK's edge over LRU is real, flat-CLOCK tuning headroom, and the footprint-latency coupling.
- **Per-policy notes** grouped under SLRU, CLOCK, plain LRU, and ghost/reuse-distance admission.


---

- finished: 2026-06-06T05:47:18+00:00
- duration: 53.46s
- status: ok
- input_tokens: 5
- output_tokens: 4056
- cache_read_tokens: 71920
- cache_write_tokens: 15425
- cost_usd: 0.14707
