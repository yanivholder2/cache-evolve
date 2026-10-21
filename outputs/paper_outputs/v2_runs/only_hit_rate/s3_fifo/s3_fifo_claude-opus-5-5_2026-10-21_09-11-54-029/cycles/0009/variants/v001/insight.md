# Variant v001 Insight

## Helped

(No measured dimension improved.)

## Hurt

- hit-rate: The survivor branch of `evict_main` now applies `main_pass_credit` (freq ≥ 2 → 1, freq 1 → 0) instead of clearing to 0. Any main entry with freq ≥ 2 now stays resident for a second hand lap, and that extra lap does not go to the keys H01 meant to favour. The counter is not reset when an entry graduates from small (`freq > 1` in `evict_small`), so every small graduate enters main with freq ≥ 2. It gets the second lap from its small-queue hits alone, even if main never hits it. Ghost re-admissions enter at freq 0 and are now sweep victims sooner relative to those graduates. They are the entries that the parent's clear-on-pass had made room for. The drop is small, about the size of the ±0.001 band the parent's gain sat just outside, and it is in the direction H01 named as the "single visited bit is optimal" outcome. The change is still isolated cleanly: only the pass rule changed, and `RefS3Fifo::evict_main` was updated in lockstep, so the exact main/freq/hand reference check still holds.

## Takeaway

On workload-a, a second lap of main retention earned by freq ≥ 2 does not beat SIEVE's single visited bit. It slightly undoes the cycle-8 gain, which supports clear-on-pass as the better main-retention rule here. In this design the "two hits" signal is mostly small-queue history that carries into main, not repeated main hits. So this experiment never really tested S3-FIFO's 2-bit idea. A cleaner test would reset or cap freq at graduation, so that only main hits can earn the second lap.
