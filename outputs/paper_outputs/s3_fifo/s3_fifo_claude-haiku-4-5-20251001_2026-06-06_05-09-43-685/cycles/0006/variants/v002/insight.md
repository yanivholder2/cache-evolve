# Variant v002 Insight

## Helped

- **per-op-time (get)**: Pre-computed frequency tracking in ghost eliminates complex frequency reconstruction on hits, reducing get p50 latency from 248ns to 175ns.
- **per-op-time (put)**: Storing (K, i8) tuples in ghost and using direct arithmetic boost (`ghost_freq + 3`) replaces heavyweight frequency logic, improving put p50 from 1088ns to 550ns and p99 from 4150ns to 766ns.

## Hurt

- **hit-rate**: The fixed boost formula `ghost_freq + 3` overshoots re-admission frequency for low-decay items and undershoots for high-decay items, causing the variant to miss re-access opportunities that baseline's adaptive logic captured, dropping hit-rate from 0.7164 to 0.7074.

## Takeaway

This variant trades accuracy for speed: storing pre-computed frequencies in ghost and simplifying re-admission arithmetic delivers substantial latency wins (49% faster puts), but the aggressive fixed-formula boost breaks the frequency-to-actual-hotness mapping that the baseline maintained. The decay threshold at -2 also prunes items before they accumulate re-accesses, whereas the baseline's immediate discard of low-frequency items allowed more opportunistic recovery. The work shows that while hot-path optimization via pre-computed state is effective, the re-admission signal must remain adaptive to workload characteristics to preserve hit-rate.
