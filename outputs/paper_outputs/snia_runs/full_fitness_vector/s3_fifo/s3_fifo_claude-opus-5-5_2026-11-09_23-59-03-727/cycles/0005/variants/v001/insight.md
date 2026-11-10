# Variant v001 Insight

## Helped

- hit-rate (hm_0, small cache only): With main at ~90% of capacity, the reuse-filtered main queue holds more of hm_0's reused keys at the 16384 size. This is the only trace/size point that improved, which fits the hypothesis's claim that the extra main space would favour hm_0.

## Hurt

- hit-rate: web_0 dropped at both sizes, and the drop is larger at 131072. That is the large-size gap the hypothesis meant to close, and it grew wider. A 10% probation window gives new keys too little time to collect the `freq > 1` hits they need to graduate. Keys with a reuse distance longer than the small queue get evicted from probation before their second access, and they enter the ghost instead of main. hm_0 at 131072 also slipped a little, so the bigger main did not pay off at the large size on either trace. The summary fell below the parent.
- per-op-time: The `put` p99 tail rose slightly while the medians held. One plausible cause is the ghost bound, which follows `main_size` and is now ~90% of capacity. A bigger ghost index plus a later `2 * main_size` compaction trigger make each compaction `retain` larger when it fires. In addition, with a small queue this short, more misses run `evict_small` → `push_ghost`. The summary metric did not move.

(metadata-size did not move: the change touches only sizing arithmetic and `with_capacity` hints, not `Entry` or the struct layout.)

## Takeaway

On the two-filter parent, cutting probation from 25% to 10% confirms the fallback branch of the hypothesis. For web_0, the probation window, not main capacity, is the binding constraint. The graduation filters already keep main clean, so extra main space adds little, while a shorter small queue costs web_0 its longer-reuse keys. The next web_0 experiments should therefore leave probation at 25% or more, or make graduation easier, rather than grow main. Any hm_0 gain from a larger main is confined to the small cache size and does not make up for web_0's loss.
