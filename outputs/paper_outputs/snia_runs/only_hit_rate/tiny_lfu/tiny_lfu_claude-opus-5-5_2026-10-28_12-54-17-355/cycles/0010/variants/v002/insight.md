# Variant v002 Insight

## Helped

- hit-rate: web_0 small (16384) went up a little and stayed above v001's floor, as the hypothesis required. At this size the fixed 2^19-`get` period is many times the capacity, so halving fires rarely compared with how long entries stay resident. It mostly clears out counts from old popularity bursts, and that lets the coldest-of-4 victim sample pick truly stale probation entries more often. Ceiling rounding keeps count-1 residents from dropping to 0, so the gate does not open to one-shot newcomers.
- hit-rate: hm_0 small was essentially flat. Halving this rarely at 16384 hardly changes the admission contest there, which fits v001's finding that hm_0 small does not respond to tie or admission tweaks.

## Hurt

- hit-rate: hm_0 large (131072) fell, and it was the biggest move in this variant. The hypothesis predicted a gain here. At large capacity the same fixed period is only a few times the capacity, so halving fires within a resident's reuse span. Each halving shrinks the frequency lead that slowly-reused warm entries in probation/protected have built up. Double counting gives a missed-then-put newcomer +2 right away, so after a halving it can match or beat a moderately warm resident. The late-hit tie rule or strict `>` then lets it displace that resident.
- hit-rate: web_0 large dropped slightly for the same reason. Halving compresses count gaps and pushes more contests toward ties or narrow wins for newcomers. That partly undoes the large-cache protection v001 gained by making ties harder to win.

## Takeaway

Using a fixed aging period, with ceiling halving and double counting kept, failed the main prediction. Neither large cell benefited from stale-count decay, and both lost ground. Only web_0 small gained, and only marginally. The likely cause is that aging and double counting work against each other: halving shrinks residents' counts while every newcomer still gets an undiminished +2. So at large capacity, where the period is close to the reuse span, warm residents lose contests they should win. Before trying aging again at large sizes, either drop double counting or tie the period to a multiple of capacity well above 4×. Simply making the period fixed is not enough.
