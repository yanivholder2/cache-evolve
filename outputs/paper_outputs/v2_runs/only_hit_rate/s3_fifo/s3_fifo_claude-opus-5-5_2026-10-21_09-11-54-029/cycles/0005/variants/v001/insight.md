# Variant v001 Insight

## Helped

(No measured dimension improved. The fitness vector reports only `hit-rate`. The expected side benefit is still unmeasured: `evict_main`'s worst-case rotation bound drops from `main.len() * 15` to `main.len() * 4`, so sweeps over a mostly hot main should be much shorter. The current fitness vector cannot confirm this because it has no `per-op-time` dimension.)

## Hurt

- hit-rate: Small regression against the parent. This disproves H01's claim that stale saturated keys were tying up main capacity. Changing `tail.freq -= 1` to `tail.freq >>= 1` takes the most from keys in the middle of the frequency range, not from saturated ones. A freq-7 key now survives 3 laps instead of 7, and a freq-3 key survives 2 instead of 3. Only freq-1 and freq-2 keys keep the same number of laps. In effect this undoes part of the 3→7 retention gain from cycle 0003 for the keys that earned it, and frees slots that go to ghost re-entries at freq 0, which are less likely to be hit.

## Takeaway

On workload-a, the per-lap credit that moderately reused keys receive is load-bearing. Shortening the lap tail with log-scale decay costs hit-rate, which means the cap-15 plateau seen in v002 did not come from stale hot keys crowding main. Halving trades a little hit-rate for a bounded sweep length (at most 4 laps). It is only worth keeping if a `per-op-time` measurement shows that `evict_main` sweep cost actually matters. Otherwise linear decrement should stay the credit-spending rule, and further retention work should target ghost re-entry priority instead of decay speed.
