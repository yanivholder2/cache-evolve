# Variant v003 Insight

## Helped

(No measured dimension improved. The diff against cycle 9 v001 changes only `const MAX_FREQ` from 7 to 3, plus its comment, the cap assertion in `freq_saturates_at_max_freq`, and one new saturation test. The `freq -= 1` carryover in `evict_one()`'s `small` branch, the `main` branch, the ghost path, `small_target` and `ghost_cap` are all unchanged. The one non-fitness gain is that freq now fits in 2 bits again, which a compact node layout could use.)

## Hurt

- hit-rate: Bringing the cap back to 3 limits every resident key, in both `small` and `main`, to at most 3 survival passes of lazy reinsertion in `main` (7 in the parent). It also limits small-to-main carryover to 2 credits. Hit rate fell by more than the tolerance H03 set for "carryover was the source." So part of the parent's gain came from the higher cap, not from carryover alone.

## Takeaway

This ablation counts against H03's "carryover alone" reading. With carryover held fixed, cutting the cap from 7 to 3 lost hit rate beyond the stated band. So the parent's higher cap was contributing, most likely through extra reinsertion passes for hot keys that keep getting hits in `main`, not through larger promotion credits (few keys build up 3+ hits in a 25-slot `small`). The 2-bit freq layout is not free. Before committing to it, a mirror ablation (cap 7, promotion reset to `freq = 0`) is needed to measure how much the carryover adds on its own. Both effects are near the noise floor on one trace at one size.
