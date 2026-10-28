# Variant v001 Insight

## Helped

- hit-rate (web_0 @ 131072, hm_0 @ 131072): No gain, but the containment worked. Both large cells match the parent exactly. So gating `door_bits` behind `capacity <= 32_768` (the same gate as `step_init`) left the large-capacity path unchanged, bit for bit, as the hypothesis required.

## Hurt

- hit-rate (web_0 @ 16384): Basically flat, a hair below the parent, when the hypothesis predicted a gain. A doorkeeper with four times the bits produced no measurable improvement in admission. So doorkeeper saturation was not what limited small-size web_0.
- hit-rate (hm_0 @ 16384): A clear regression, against the "neutral or better" prediction. With fewer false positives, a key's first touch now often sets only door bits instead of also going into the count-min counters. Keys that recur moderately within a reset period therefore get lower estimates. They lose more ties against established victims, and those keys are the useful reuse on hm_0.
- hit-rate (summary): Lower than the parent. The whole drop comes from the two small cells, mainly hm_0.

## Takeaway

A saturated doorkeeper was not a broken filter. It worked like a roughly uniform +1 on every first touch, and a uniform bonus barely changes the candidate-vs-victim ordering. Making the doorkeeper more accurate at 16_384 removed that bonus unevenly. It hurt moderate-reuse keys on hm_0 and did nothing for web_0's one-shot keys. A bigger doorkeeper should not be tried again as a way to improve small-size web_0. The remaining limit there is probably elsewhere, for example in the admission tie-break, the `reset_at` period, or how the window and main areas are split.
