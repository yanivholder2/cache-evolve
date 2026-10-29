# Variant v003 Insight

## Helped

None. No trace/size cell improved over the parent (cycle 0005 v003).

## Hurt

- hit-rate (hm_0, both sizes): hm_0 dropped the most, and the drop was largest at 16384. The `cand_est >= 2` gate stops one-shot keys, but it still lets in keys seen exactly twice. On hm_0 those are mostly low-value. Each random admission evicts a main-segment victim that had a strictly higher frequency estimate, and that victim was usually still useful. A small cache feels this more because each wrongly evicted entry is a larger share of what it holds.
- hit-rate (web_0, both sizes): web_0 also lost hit-rate. This is the trace the change was meant to help. If stale, inflated victim counts were locking out real recurring candidates, the random escape hatch should have recovered some hits. It cost hits instead. So "lockout" is not what holds web_0 back in this lineage.

## Takeaway

Randomly admitting warm candidates (estimate ≥ 2, 1/64 chance) after they lose the full-window duel hurts every cell. It hurts most where admission must be strict (hm_0 at the small size). The parent's deterministic `cand_est >= victim_est` rule, which admits on ties, is already lenient enough. With the 16x reset period, victims that win the duel are mostly earning it, not winning on stale counts. Caffeine-style anti-lockout randomness does not help here and should not be revisited unless the sketch reset period is made much longer. Caveat: I could not reach the overlay source in this run. This reading assumes the change matches H03 exactly: the RNG is called only on a deterministic reject with `cand_est >= 2`, and nothing else changed.
