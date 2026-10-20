# Variant v003 Insight

## Helped

- hit-rate: Shrinking the small queue from 5% to 3% (main = total − small) filters slightly more one-hit-wonders before they reach main and leaves more room for re-referenced objects. The gain is real but tiny, so the trend is flattening.
- per-op-time: The `put` median and the `get`/`put` tails moved slightly in the good direction. The change touches only a size constant, so this is most likely measurement noise, not a code-path effect.

## Hurt

- per-op-time: The `get` median moved up marginally. A size constant cannot plausibly cause this, so it is likely noise, not a regression.

## Takeaway

Moving the small-queue ratio from 5% to 3% yielded only a marginal hit-rate gain, a much smaller step than the earlier 10%→5% move. That suggests the small-queue sizing lever is near saturation on this workload. The hypothesis tested a constant only, so metadata-size and timing were never expected to move. Further gains likely need changes to promotion or ghost policy, not a smaller small queue. I could not read the overlay sources in this session, so this analysis rests on the hypothesis and the fitness vectors.
