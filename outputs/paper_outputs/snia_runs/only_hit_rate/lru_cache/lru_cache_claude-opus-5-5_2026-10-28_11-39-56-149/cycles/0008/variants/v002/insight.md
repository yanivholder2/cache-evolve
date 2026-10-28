# Variant v002 Insight

## Helped

- hit-rate (web_0 @ 131072): Setting `prot_pct` to 75 above 32_768 makes the probation segment of the main area longer. web_0's medium-range reuse blocks then stay resident long enough to get their second hit and be promoted, instead of being evicted from a short probation queue. This is the same mechanism that made cycle 0007 v002's global shrink help this cell, and the milder 75% split keeps most of that gain.
- hit-rate (summary): Both small-size cells match the parent exactly, as expected, because `main * 80 / 100 == main * 8 / 10` leaves capacities ≤ 32_768 unchanged. The web_0 gain at the large size is bigger than the hm_0 loss there, so the net result is a strict improvement, as the hypothesis predicted.

## Hurt

- hit-rate (hm_0 @ 131072): A marginal loss. hm_0's reuse set already fits in the protected segment at this size. Cutting protected from 80% to 75% demotes a few genuinely hot entries into probation, where scan traffic sometimes evicts them. The hypothesis expected this cost, and the cost is much smaller than the one the global shrink in cycle 0007 v002 caused.

## Takeaway

Like `step_init`, the right protected/probation split depends on cache size. Gating it at the same 32_768 threshold keeps the large-cache web_0 benefit of a longer probation and avoids the hm_0 @ 16384 regression that a global shrink caused. Both size-gated levers now cost a little on hm_0 @ 131072 in exchange for web_0 @ 131072. So more large-size gains probably need a lever that tells scan-heavy traces apart from medium-reuse traces, such as adapting `prot_pct` at runtime with the climber or duel, rather than another static size gate.
