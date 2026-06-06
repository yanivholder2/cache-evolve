## Recurring wins

- Changes that *remove* hot-path work pay for themselves; changes that *add* a branch or store rarely do. The biggest movers — the fast integer hasher, the metadata fold to a 24-byte node, gentle ghost admission, and flat CLOCK dropping the recency list — each cut work and so came free or near-free on multiple axes.
- The size/representation *mechanism* is reliably cheap: spare-high-bit flags, `u32` sentinel links, and packing the segment flag into a link consistently hold the node-byte target with little or no latency tax, especially with centralized cast helpers. When these variants lost, the loss was always the encoded *policy*, never the encoding.
- Orthogonal levers stack: the policy axis (segmentation/CLOCK) and the per-op-time axis (cheap hasher, surgery skips) have repeatedly improved together with neither paying for the other.
- Small geometry nudges can buy hit-rate for free even though large ones don't — effects here are magnitude-dependent, not categorical.
- A genuinely structural change can reach room that micro-edits never could (flat CLOCK beat the long-standing SLRU champion on latency and footprint at equal hit-rate after several "at the frontier" cycles).

## Recurring failures

- Admission cleverness keeps losing on this short-reuse trace: ghost/reuse-distance re-admission and two-hit probation gates either fire too rarely to move recall or drain genuine repeaters before their second hit, dropping hit-rate while adding bookkeeping. Treat as tried-and-regressed.
- Latency micro-opts aimed at non-bottlenecks consistently regress via codegen damage (register/code-size pressure, unsafe risk): bounds-check elision, relink/store fusion, and probe-skipping get caches all attacked work that was already free. The real hot-path cost is hashing, link surgery, and cache-line traversal.
- Skipping the HashMap probe never pays — the zero-sized xor-multiply hasher plus arena index is cheaper than any guard layered in front of it.
- Bare data-dependent branches on the universal hot path mispredict and regress; position-gated skips pay only where the predicted position is actually common (inside the SLRU protected MRU, not on flat LRU).
- Large geometry jumps (90/10, 15/16) starve the fast path; a sharp cliff sits just above the 7/8 protected cap.

## Open questions

- Does the SLRU protected/probation split add anything over flat CLOCK on this trace, or was the whole segmented lineage carrying cost without payoff here?
- Is CLOCK's slight edge over exact LRU real signal for short-reuse, or noise?
- Where is remaining flat-CLOCK headroom — hand placement, sweep bound, fresh-key admission — now that the segmented levers are superseded?
- Does growing the cache struct footprint carry a real latency tax worth treating as a constraint, or is the footprint-to-p50 coupling non-causal (the `u32`-narrowing control refuted it)?

## Per-policy notes

- SLRU: long-reigning champion across the middle cycles; hit-rate has two levers — retention (geometry) and admission. Its protected-MRU fast path is already write-free, so adding any per-hit store (CLOCK bit, ghost bookkeeping) backfires there.
- CLOCK: wins only on a flat single list, where there is no write-free MRU branch to lose; one reference bit matches LRU hit-rate while removing the recency list, cutting latency and footprint together.
- Plain single-list LRU: the surgery-skip fast path does not generalize here — the touched node is rarely the tail, so the position guard mispredicts.
- Ghost / reuse-distance admission: unconditional promote-to-protected overshoots and evicts hot keys; gentler gating helped once (Cycle 0004) but did not reproduce on the short-reuse workload — trace/size-contingent, now effectively refuted.
