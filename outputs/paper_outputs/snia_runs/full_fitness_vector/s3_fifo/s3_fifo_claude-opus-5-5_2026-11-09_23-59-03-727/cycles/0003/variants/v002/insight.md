# Variant v002 Insight

## Helped

- hit-rate (hm_0): Setting `freq = 0` when an entry graduates in `evict_small()` stops entries that were hit in small but are cold in main from spending up to three extra `evict_main` sweeps on credit they earned earlier. Main slots go back sooner to entries that are actually reused, and hm_0 improves at both cache sizes, as H02 predicted.
- hit-rate (web_0): web_0 held and edged up at both sizes. The reset alone does not hurt this trace. Cycle-2 v002's web_0 loss must therefore come from the other change it bundled in, lowering the graduation threshold from `freq > 1` to `freq >= 1`, which this variant left alone.

## Hurt

- per-op-time: The `get` median, which is the summary, rose even though `get` is byte-for-byte identical to the parent, and the only new work is one store on the rare promotion path. The `get` shift is therefore not caused by the code; most likely the run environment changed. The bigger `put` p99 increase could be partly real. With carried-over frequency gone, more main entries are evictable sooner, so main evictions and `push_ghost` calls (and the occasional ghost `retain` compaction) may happen more often. That has not been verified, and environment noise is just as likely.

## Takeaway

This isolation experiment gives a clean answer. Resetting frequency on promotion from small to main is a hit-rate win on its own: hm_0 improves and web_0 holds. That makes the looser `freq >= 1` graduation threshold the cause of cycle-2 v002's web_0 regression. Future variants should keep the reset and the `freq > 1` threshold. Before reading anything into per-op-time changes, they should re-check timing under a fixed environment, because this one-line policy change cannot explain a slower `get`.
