# Variant v002 Insight

## Helped

(none — no trace/size cell improved over v003)

## Hurt

- hit-rate: At size 16384, both web_0 and hm_0 fell clearly below v003. The halving period there is 163 840 `get`s (10×capacity), so the sketch is aged many times per trace. Each `halve()` shrinks the frequency lead that long-lived main-LRU residents had built up. Window candidates with a short recent burst then win the strict `>` contest more easily and displace keys that would have been re-referenced. The adaptive window was already giving newcomers room to prove themselves, so the extra aging adds churn instead of letting stale counts decay usefully.
- hit-rate: At size 131072, web_0 and hm_0 both dipped by small amounts. The period is 1.31M `get`s, so halving fires only a few times per trace. That is too rarely to change admission decisions much in either direction, so the predicted web_0 gain did not appear.

## Takeaway

Counting the aging period only on `get` (and flooring it) fixed the "ages twice per miss" issue that H02 blamed for cycle 0002's hm_0 small-size regression. Even so, hm_0@16384 still regressed, and this time web_0 regressed too. That fails both of H02's refutation tests. Cap-free sketch halving and v003's adaptive window do not stack. Once the hill climber has sized the window, periodic halving mostly wears away the main region's protection: it hurts at small capacity, where halvings are frequent, and does nothing at large capacity, where they are rare. Future aging attempts on this lineage should either be tied to the climber's state or be dropped in favor of a structural change (e.g. SLRU main). Tuning the period alone is not enough.
