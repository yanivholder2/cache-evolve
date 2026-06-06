# Variant v001 Insight

## Helped

- hit-rate: Replacing the single-slot `is_hottest` test with the `len/4` `is_in_promotion_window` gate lets a few more warm target-tier items climb into probing, nudging hits up — but only marginally, within trace noise.

## Hurt

- per-op-time: The wider gate makes `promotion_get` fire the `remove_at` + `promote_to_probing` tier cascade for far more keys per `get`, raising get and put p50/p99 over the parent's hottest-only path.

## Takeaway

Stacking v003's top-quarter window onto v001's slim+MRU base did not compound: `InsertPosition::Back` already places fresh hits at the MRU/back end where the window opens, so widening the gate mostly re-promotes mass MRU had already captured — paying real promotion-cascade latency for a hit-rate gain indistinguishable from noise. This answers cycle 0002's open question 1: the gate and MRU placement are substitutes, not additive wins.
