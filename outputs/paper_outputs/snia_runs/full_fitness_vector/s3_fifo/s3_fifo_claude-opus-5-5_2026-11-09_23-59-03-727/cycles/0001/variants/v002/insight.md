# Variant v002 Insight

## Helped

- hit-rate: Ghosting main-queue victims helps where it was supposed to: on hm_0 at the large cache size, which has a big recurring working set, keys aged out of main now come back straight into main instead of serving probation in the small queue again. web_0 at the small size also gained a little. The aggregate gain is marginal.

## Hurt

- hit-rate: hm_0 at the small size and web_0 at the large size regressed slightly. The ghost queue is still capped at `main_size`, so main victims now compete for ghost slots with demoted small-queue keys. That pushes out the history that previously let a reused small-queue key go directly into main, and it partly cancels the gains.

## Takeaway

Widening the ghost to cover main victims without enlarging it mostly moves ghost capacity from small-queue demotees to main-queue victims. It pays off only when the main-victim reuse distance fits inside the existing ghost bound, so the net hit-rate effect is close to neutral. A follow-up should either size or split the ghost separately for main victims or limit ghosting to main victims that once had a nonzero frequency. metadata-size did not change because the patch adds no fields. per-op-time improved sharply, but this change can't explain it: `get` is untouched and the ghost bound is the same. Treat it as a difference in the measurement environment and re-check it against a baseline run on the same machine before giving this variant credit.
