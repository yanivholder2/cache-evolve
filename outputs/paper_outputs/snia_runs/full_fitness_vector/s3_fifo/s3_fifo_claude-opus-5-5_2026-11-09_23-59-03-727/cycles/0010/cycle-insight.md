# Cycle 0010 Insight

## Cohort outcome

All three variants were single-change children of cycle-9 v001. v001 narrowed the size-gated small share from 1/5 to 1/6. v002 replaced the rare ghost `retain` with a bounded stale-tail trim on every `push_ghost`. v003 capped the main sweep at 32 reinsertions, then force-evicted. Only v001 gained: hit rate rose slightly at 16384 on both traces, mostly hm_0, and 131072 was identical by construction. Its small p50/p99 drift is probably noise. v002 kept hit rate identical but made the `put` tail worse. v003 made both the `put` tail and hit rate worse in every cell.

## Patterns

- The size-gated split is still moving in one direction. 1/4 → 1/5 → 1/6 each gave a tiny gain at 16384. Cycle 5's global 1/10 hurt web_0, so the peak is probably between 1/6 and 1/10. The returns are shrinking.
- Both timing variants failed for the same reason: they added work to the common path in order to remove rare work. v002 put an unconditional cold-memory probe on nearly every miss, the same mistake cycle 7's ring made. The `retain` it was meant to amortize fires too rarely to show up at p99, and the full-ghost pop loop already cleans stale tails.
- v003 contradicts the "sweep length drives the `put` tail" claim from cycles 8–9. Sweep steps are cheap. Force-evicted entries are always reused, so the cap moved work into `push_ghost` and its compaction, which have heavier tails. The cap also bound often enough to evict hot keys, at both sizes. That disproves the idea that 131072 is insensitive to every policy change. It is only saturated against changes that leave the retention of hot keys alone.
- Every timing hypothesis since cycle 7 has been inferred from code, not measured. The cohort keeps guessing wrong about what actually sets the `put` p99.
- Per-entry metadata stayed flat across all three variants.

## Open questions

- What actually sets the parent's `put` p99: ghost insert and compaction, hash-map rehash, sweep steps, or benchmark noise? Nobody has profiled it.
- Where between 1/6 and 1/10 does the 16384 share peak, and is web_0 the trace that loses first?
- Is the ~1 ns p50 drift on policy-only diffs reproducible, or is it run-to-run noise?

## Continuation

Promote cycle-10 v001 (the 1/6 split) as the parent. Run one final share probe at 1/7 or 1/8. After that, treat hit rate as converged. Drop sweep caps, forced eviction, and any per-miss trim or liveness probe, even ones that exit early. On timing, require hypotheses to name a measured source of the `put` p99, ideally ghost insert/compaction or rehash, before changing code. Never add unconditional work per miss, and give no timing credit for ~1 ns shifts.
