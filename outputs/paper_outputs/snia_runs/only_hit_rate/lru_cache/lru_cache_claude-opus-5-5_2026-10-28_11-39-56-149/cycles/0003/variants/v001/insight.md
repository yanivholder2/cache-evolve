# Variant v001 Insight

> Source note: this session could not list or read the overlay sources, manifest, or other eval artifacts, only `eval/fitness-vector.json`. The reasoning below about how the change works comes from the H01 spec and the parent's code and insight. It is not checked against the code that was actually landed.

## Helped

- hit-rate (web_0 @ 16384): This was the cell H01 aimed at, and it improved. The first climb step is `+capacity/16`, which grows the window from about 160 entries to over 1k. That lets more of web_0's short-range second accesses get re-referenced inside the window, before they face the narrow, often-halved sketch that v002's insight blamed for this shortfall. The gain is small next to that diagnosis, so the window size was only part of the problem. The sketch size and reset period, and the miss-then-put double increment, are still the likely causes of the rest.
- hit-rate (hm_0 @ 16384): This also improved, by about as much as web_0 at this size. At small capacity, `sample_period = max(1000, 10*capacity)` allows many climb rounds per trace, so the climber can try a larger window, keep it while it pays, and back off when hit-rate drops. The tie-admitting gate stayed unchanged, so hm_0's frequency protection survived the bigger window.
- hit-rate (hm_0 @ 131072): A marginal gain. It shows that adaptive sizing kept hm_0's gains at the large size, as H01 required, and did not erode them.

## Hurt

- hit-rate (web_0 @ 131072): This was v002's best cell, and it slipped. At this size the opening step is +8192 entries, a jump from about 1.3k to about 9.5k that takes window space from the main SLRU. `sample_period` is 1.31M gets, so the climber gets few chances in one trace to undo that move. Also, while the window grows, each insert evicts the main victim with no duel. That temporarily bypasses the TinyLFU gate, which v002 showed was helping web_0 at this size.

## Takeaway

Adaptive hill-climbed window sizing pays off where the climber gets many sample rounds (small capacity on both traces). Where samples are long and the first step is large (`capacity/16` with a `10*capacity` period at 131072), it hurts, because it moves v002's well-tuned 1% split away from its optimum and rarely comes back. The next refinement should scale climber speed separately from capacity: a smaller initial step or a capped `sample_period` at large sizes. Separately, fixing the sketch issues should go after the remaining web_0 @ 16384 gap, which window size alone did not close.
