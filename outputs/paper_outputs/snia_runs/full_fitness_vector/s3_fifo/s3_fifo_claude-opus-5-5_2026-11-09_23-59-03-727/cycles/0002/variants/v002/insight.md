# Variant v002 Insight

## Helped

- hit-rate (hm_0 at both sizes): with the `freq >= 1` threshold, hm_0 keys that are reused once during probation now graduate to main and stay resident. Before, they went to the ghost queue and had to take another miss to come back, so hm_0 gained a little more on top of v001's gains.

## Hurt

- hit-rate (web_0 at both sizes, including the targeted 131072): web_0 dropped slightly, which is the opposite of what H02 predicted. Two things changed at once. Promotion now resets freq to 0, so keys that were hit several times in small used to bring their count into main and get second chances in `evict_main`, and now they land as the first eviction candidates. Also, the looser threshold puts more single-reuse keys into main, and these crowd out longer-lived web_0 entries.
- per-op-time: get and put are slightly slower, but the change only swaps one comparison and adds one store on the eviction path, and `get`'s lookup is untouched. Treat this as benchmark noise, not a real cost.

## Takeaway

H02 targeted the wrong mechanism for web_0. v001 blamed the web_0@131072 loss on keys being evicted from small *before* their second reference, which means at freq 0. Changing `> 1` to `>= 1` only affects keys that already had exactly one hit, and freq-0 keys still go to the ghost queue as before. So the change only helps hm_0, a trace that is sensitive to promotion. H02 also changed two things together (looser threshold and freq reset on promotion), so we can't tell which one caused the web_0 regression. A follow-up should test the reset separately, or test a longer probation window directly, before deciding the threshold is the right lever.
