# Variant v001 Insight

## Helped

- hit-rate: hm_0 at the small size was the only cell that moved up, and only marginally. Periodic halving adds some recency. It lets stale probation entries that were once warm decay enough to lose the contest to fresh window candidates. But the effect is too small to count as the "real recency" gain the hypothesis predicted for hm_0.

## Hurt

- hit-rate: web_0 at the small size took the clearest loss. Single counting already keeps sketch counts low (a miss-then-put now adds 1, not 2). `halve()` then shifts every count-1 key to 0, and that includes the cold-but-recurring probation tail. Under the strict `cand_freq > victim_freq` test, any window candidate seen once now beats such a victim. `sampled_main_victim()` takes the minimum of 4, so it almost always finds a zero right after a reset. The free-admission problem from halving alone is back, just one count lower. On a frequency-skewed trace, those post-reset admissions push out entries web_0 would have hit again.
- hit-rate: web_0 and hm_0 at the large size slipped slightly. The halving period grows with capacity (10× capacity in `get` calls), so there are fewer resets over the trace. At these sizes contests also decide a smaller share of hits, so the damage is small. It is still a net loss: wiping out long-run frequency history costs more than the recency it adds.

## Takeaway

Pairing single counting with halving does not keep the admission bar level. Integer `>>= 1` rounds a victim's count-1 down to 0, while a newcomer's count-1 survives until the next reset. So after every reset, the coldest-of-4 victim sample still gives candidates a systematic edge, and web_0 pays for it, most of all at the small size. hm_0 barely moved, which supports the parent's finding: hm_0's loss comes from where entries are placed in probation, not from how frequencies are counted or aged. If halving is tried again, it needs either a non-strict or floored contest (for example, a victim at 0 counts as 1), or a much longer reset period. Halving should not be expected to fix hm_0 on its own.
