# Variant v002 Insight

## Helped

- hit-rate: hm_0 at the small size rose a little more above the parent. Because of the floor, one-hit wonders that arrive at estimate 2 still lose ties, so the stickier-incumbent effect that conservative update gave hm_0 is kept. Letting repeat-referenced window candidates win ties at 3 and above adds a small recency bonus: when two keys have the same frequency, the one referenced more recently takes the slot, and that suits hm_0 when the cache is small. hm_0 at the large size did not move in any meaningful way.

## Hurt

- hit-rate: web_0 fell again at both sizes, and by more at the large size. This is the opposite of the "recovery" the hypothesis predicted. The change does exactly what was intended. Ties at estimate 3 or more now go to the window candidate, and the incumbent it displaces is the coldest of the 4 least-recent probation entries. On web_0, those equal-frequency incumbents seem to be worth more than the candidates that replace them. A likely reason is that a candidate at estimate 3 has had only one real re-reference beyond the double-counted miss-then-`put`, while a probation resident at the same estimate has already survived in main. Swapping them churns main without improving its quality. The larger drop at the large size fits this reading: a bigger probation segment holds more warm residents at small counts, so more ties at or above the floor are decided against them.

## Takeaway

Adding a candidate-favouring tie-break above estimate 3 did not undo the web_0 loss that conservative update caused. It made that loss deeper. This is evidence against the parent insight's diagnosis that warm newcomers losing low-end ties were the cost. If ties matter for web_0 at all, the ones that matter are at estimate 2, which the floor deliberately leaves to the incumbent, or the loss comes from somewhere else in how conservative update reshapes the counts. In the current admission path, equal frequency is not a good enough reason to replace a main resident with a window candidate. The next experiment should either test the estimate-2 tie on its own or drop conservative update, rather than push further on the tie rule above the floor.
