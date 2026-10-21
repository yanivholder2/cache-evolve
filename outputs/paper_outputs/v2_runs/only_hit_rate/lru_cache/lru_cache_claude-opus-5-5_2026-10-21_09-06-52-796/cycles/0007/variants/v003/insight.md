# Variant v003 Insight

## Helped

(No measured dimension improved. The fitness vector reports only `hit-rate`, and it went down slightly from the parent.)

## Hurt

- hit-rate: Admitting ghost-hit keys with `freq = 1` instead of 0 means each one survives its first trip past `main`'s CLOCK hand. On that trip it is decremented and reinserted, and the scan moves on to evict the next `freq == 0` resident instead. Those victims are mostly keys promoted from `small`, whose freq `evict_one` still resets to 0, and older `main` keys that have used up their credits. A ghost hit is evidence of only two references, with nothing yet observed while resident. So the free credit holds ghost arrivals that are never referenced again for one extra full `main` cycle, roughly 1,950 slots of churn at size 2048. The cost is paid by entries with equal or better reuse evidence. The new test `ghost_admitted_key_survives_first_main_scan` encodes exactly this trade: key 3 was hit while in `small`, yet it is evicted ahead of ghost-admitted key 1.

## Takeaway

The parent's insight said ghost-path keys arrive "with no protection". That turned out not to be a weakness worth fixing on workload-a. With a short `ghost_cap = capacity / 8` window, a ghost hit is already a strong enough filter to get a key into `main`, and `main`'s own reinsertion loop gives protection on time to keys that are actually reused. An unearned credit just makes `main` keep false positives longer. It also inverts the evidence ranking: ghost admissions now outrank `small` promotions, though `small` promotions showed a hit while resident. If retention for ghost admissions is revisited, the `small`-promotion path has to be considered alongside it. For example, give promoted keys a matching credit, or reserve the credit for repeat ghost hits. Changing only one admission path's starting freq skews which path loses in `main` and does not add any real protection.
