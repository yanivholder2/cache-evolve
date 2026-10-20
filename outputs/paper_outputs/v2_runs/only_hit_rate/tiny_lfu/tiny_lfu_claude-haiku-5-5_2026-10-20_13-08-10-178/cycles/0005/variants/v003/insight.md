# Variant v003 Insight

## Helped

- None: the only measured dimension, hit-rate, moved down, and this variant's fitness vector reports no other dimensions.

## Hurt

- hit-rate: Periodic global halving of every sketch counter (`CountMinSketch::halve`, triggered from `record` once `samples` reaches `10 * capacity`) discards frequency history that the admission check `incoming_freq > victim_freq` relies on, so keys that were recently warm get compared against a sketch that has been shifted down and lose their admission edge.

## Takeaway

Adding a sample-window halving at 10x capacity, with no other change to admission, reduced hit-rate on workload-a at size 2048, so this trace still needs the sketch's accumulated frequency history at this capacity and aging on this window is net harmful. Because the halving is global and happens on a fixed access count, a single shift can invalidate the relative ordering the admission test depends on; any future aging variant on this subject should be tested against a window size or trigger that is not tied only to access count.
