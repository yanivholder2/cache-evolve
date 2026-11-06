# Cross-Cycle Insights (cycles 0001–0010)

## Recurring wins

- **Put cost tracks hash and bookkeeping count.** Confirmed in six or more cycles. Single-`Entry` no-clone put (c4), cheap Fx ghost tags on the vacant path only (c6), a denser ghost table (c9) and a cheaper folded-multiply hasher (c10) all lowered put. Every extra hash added put cost: ghost `HashSet`, SipHash sketch, separate `hash_one`, a second hash for remove-before-insert, SipHash on backward shift.
- **Packing policy state into existing bits keeps metadata flat.** Examples: `u32` indices, a segment flag in a link high bit, a counter or flag in padding or `prev`. Settled since c2, and now the default way to add policy state.
- **Selective, ghost-evidenced reprieves beat uniform window changes.** A probation-only second chance keyed on a ghost hit helped web_0 where eager ghosts and strict gates hurt it (c4–c5). At both sizes it beat a static wider probation, which only traded hm_0 hits for web_0 hits (c5, c8).
- **Size-gating hit-rate mechanisms isolates their effect.** Gating avoided the 131K hm_0 loss (c5). Ghost width at 131K worked as a reuse-distance filter: /8 was too short, /2 leaked, /4 came out about best (c6–c8).
- **Deterministic (zero- or fixed-seeded) ghost hashing** made small hit-rate deltas trustworthy (c6–c7).

## Recurring failures

- **Get p50 has resisted every lever for ten cycles.** Ruled out as causes: list/relink work, flag bit-ops, hasher speed, lookup cache lines (custom table), forced inlining, SoA link split, neighbour density. Ghost footprint looked causal in c8 and was downgraded in c9. The median often moves when get code is byte-identical, so any single-run delta of that size is noise.
- **Get-p99 "fixes" don't hold up.** Unrelated changes "fixed" it twice (c7 and c10). Trading the median for the tail has lost under the p50 summary.
- **A paired, repeated noise-floor run has never been done.** It was requested in c5 through c9 and still hasn't run. Every per-op claim so far comes from single runs.
- **Kinds of history state that failed:** full-key ghosts with eager promotion to protected; miss-only, tie-rejecting TinyLFU gates; "was once protected" flags (pre-eviction evidence); stealing ghost tag bits for policy state (more collisions cost web_0).
- **Tuning that failed:** online hill-climbing of the SLRU split, static probation-share retunes at either size, more ghost-width tuning, CLOCK on the protected segment, GCLOCK with decrement-by-one aging on the eviction path.
- **Latency hacks that failed:** a custom scalar index table in place of hashbrown, a bitset ghost with per-put cursor aging, `#[inline(always)]`, any added work on `on_hit`, and growing the header past one cache line.

## Open questions

- What is the actual run-to-run noise floor for get/put p50/p99? It gates every latency conclusion.
- Does the folded-multiply hasher stack cleanly onto the ghost/escalation lineage (c9 v001 → c10 v002)?
- Can the second-return (escalation) flag live outside the ghost tag, in a side bit or tiny table, so web_0 recovers?
- At 131K, is a second-return signal reachable with a capacity/4 ghost, or do returners need a wider window? Does the ghost still pay for itself there compared with no ghost?
- Would a smaller (not split) node or better probe locality move get p50, given that the hit entry's own lines are the last suspect left?
- `metadata-size` doesn't count the heap ghost table. Should it?
- Can a bitset ghost age with no per-put work, for example by flipping a generation on a double buffer?

## Per-policy notes

- **SLRU (lineage base since c2):** flag-packed on `u32` nodes. It improves every trace at both sizes, and its cost sits in put (promote/demote relinks). The current base lineage is no-clone single-`Entry` put, then a size-aware fingerprint ghost reprieve (full at 16K, capacity/4 at 131K, 8-bit tags), then second-ghost-hit escalation to protected (helps hm_0 at 16K and costs web_0 through tag collisions).
- **CLOCK / GCLOCK (c1–c2, c5):** lazy promotion shrank metadata but didn't lower get p50. Its hit-rate gain was narrow (mostly hm_0) and the put sweep made tails worse. CLOCK on SLRU's protected segment didn't help. The CLOCK family is dropped unless aging moves off the eviction path.
- **TinyLFU / ghost-admission variants (c3):** harmful in their bare forms. The only history variant that has paid off is the gentle ghost reprieve inside probation.
- **Traces:** hm_0 rewards longer probation and frequency signals. web_0 is very sensitive to how aggressive admission is and to ghost collisions, especially at 131K. Levers that help one trace usually hurt the other unless they act on ghost evidence.
