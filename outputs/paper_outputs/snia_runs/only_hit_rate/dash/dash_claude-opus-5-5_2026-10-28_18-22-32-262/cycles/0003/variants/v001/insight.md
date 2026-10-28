# Variant v001 Insight

## Helped

- hit-rate (128K, `hm_0`): This was the largest gain, though still small. Items demoted from a normal bucket now enter the stash with `lfu_counter = STASH_PROMOTION_HITS - 1`, so they go back to their target on the first re-hit. On this trace, items pushed out of the larger target buckets are often reused soon, so returning them one hit earlier converts a few stash hits into target residency.
- hit-rate (128K, `web_0`): Only a token gain. The change does what the hypothesis asked: demoted items no longer wait two hits. But it recovers almost none of the regression that the cycle 0002 insight blamed on the counter reset.
- hit-rate (16K, `web_0`): Essentially flat. At this size, fresh puts still need two hits, and those keep the filtering benefit. The one-hit return for demoted items adds almost nothing.

## Hurt

- hit-rate (16K, `hm_0`): A slight loss. When a demoted item promotes on one hit, it re-enters a scarce target slot right away. That pushes another target resident down the `target -> stash` cascade, which is the kind of swap the two-hit gate was suppressing at small sizes. Demoted items now bounce between target and stash more readily.

## Takeaway

Giving demoted items a one-hit return (`lfu_counter = 1` on demotion) technically pushes the mean just past the parent's. But every trace/size cell moved only marginally, and `web_0` 128K did not recover. By the hypothesis's own test, the counter reset was not the main cause of that regression. The remaining cost most likely comes from gating fresh `put` arrivals at large sizes, where target space is plentiful. The demotion counter is a near-neutral knob: it trades a small 128K gain against a small 16K loss on `hm_0`.
