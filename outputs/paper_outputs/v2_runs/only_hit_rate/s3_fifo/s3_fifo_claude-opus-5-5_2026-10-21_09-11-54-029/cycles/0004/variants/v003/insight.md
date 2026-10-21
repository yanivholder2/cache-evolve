# Variant v003 Insight

## Helped

(No measured dimension improved. The fitness vector reports only `hit-rate`, and it regressed.)

## Hurt

- hit-rate: Ghost hits now get a second probation in `small`, and that probation is far shorter than the reuse distance that produced the ghost hit. A key qualifies as a ghost hit when it is re-accessed within about `main_size` (1946) small-queue evictions of leaving small. The new probation then asks it to be hit again within about `small_size` (102) insertions. Keys whose reuse gap was long enough to fall out of small the first time are the keys least likely to show a second reuse in a window roughly 19× smaller. Most of them reach the small tail still at freq 1, go back to `ghost`, and keep cycling between `ghost` and `small` instead of settling in main. Each miss on such a key is a miss that the parent turned into a main-resident hit.
- hit-rate (secondary): Re-admitted ghost keys now share the 102-slot small queue with cold inserts. Every ghost re-entry triggers an `evict_small`, so cold keys spend less time in small and get fewer chances to collect the two hits they need to graduate. Ghost hits used to go straight to main without touching small, so the parent's small-queue filter had no such competition. This variant weakens admission for both kinds of key.

## Takeaway

On workload-a, a ghost hit is already the reuse signal that matters. Requiring a second hit inside the small queue's FIFO window tests reuse on the wrong time scale: that window is about `small_size` insertions, while ghost reuse happens over about `main_size` insertions. The probation therefore rejects most keys that the ghost correctly identified. This is the outcome the hypothesis named as its falsifier. The fact that the loss is large suggests that sending ghost hits directly to the front of main is the right re-admission point. A future change to re-admission should match its probation length to the ghost horizon, for example a freq-0 main insertion, not to the small queue.
