# Variant v002 Insight

## Helped

- hit-rate: New items go in with the reference bit cleared (`lfu_counter = 0`), so the CLOCK sweep evicts one-touch keys before anything that has been hit. That gives some scan resistance that strict move-to-back LRU lacks. It slightly beat LRU on three of the four trace/size points, so the hypothesis's fear of losing hit rate did not come true.

## Hurt

- per-op-time (put): A put into a full bucket now runs a branchy loop over 40-byte items, one at a time, clearing bits until it finds a slot whose bit is 0. When a bucket is hot and most bits are set, that loop can cover up to two full passes of a 128- or 1024-wide bucket. The baseline instead did one `remove(0)`, an optimized bulk memmove. Clearing the bits also writes to every cache line the sweep touches. Both p50 and p99 got worse, and p99 got much worse.
- per-op-time (get): The hit path is now a single store, but get p50 barely moved. So the move-to-back memmove was never the main cost of a get; the linear `get_position` key scan over the wide bucket probably is. Get p99 also got worse even though the get path adds no work. One likely cause is that items no longer get reordered by recency, which changes where hot keys sit for the scan. This is a guess: the eval artifacts do not show the cause.
- per-op-time (summary): The summary is get p50 only. Its tiny improvement hides the put regression and both tail regressions.

## Takeaway

At these bucket widths, replacing LRU's memmove with a CLOCK hand does not speed things up. The per-item sweep on eviction costs more than the bulk copy it replaces, and the get path's cost is in key lookup, not reordering. CLOCK's real gain here is a little hit rate from favouring referenced items. A follow-up that wants speed should target the linear `get_position` scan, for example with a per-bucket key index or tag array. It could also put a hard limit on how far the sweep may go, and keep the clear-on-insert reference bit, which is what earned the hit-rate gain.
