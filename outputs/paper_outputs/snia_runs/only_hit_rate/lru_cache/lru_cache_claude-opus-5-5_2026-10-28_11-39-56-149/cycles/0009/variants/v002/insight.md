# Variant v002 Insight

## Helped

- hit-rate (hm_0 @ 16384): Doubling `reset_at` to 32 × capacity lets warm keys build counts over twice as many accesses before each halving. Their counts stay above the count of one-shot scan candidates for longer, so the TinyLFU duel rejects scan keys more reliably. This is the strict one-shot filtering the hypothesis expected, and it confirms the inverse of c4's result that faster aging hurts this cell.
- hit-rate (web_0 @ 16384): This cell gained more than hm_0 @ 16384. web_0's reuse distances span several cache-fills, so under the 16 × period a key's count was often halved before its next reuse. The longer memory keeps that history across the gap, and those keys win the duel against the probation victim more often.
- hit-rate (summary): Both large-size cells match the parent exactly because the `capacity <= 32_768` gate leaves `reset_at` at 16 × capacity there. The whole gain therefore comes from the small-size lever alone, with no trade-off.

## Hurt

## Takeaway

At small sizes, longer frequency memory helps both traces: the parent's 16 × capacity aging period was too short when capacity is tight, and doubling it gave a gain on both small cells with no cost. Unlike the earlier `prot_pct` and `step_init` gates, which traded hm_0 against web_0, this lever helped both small cells together. The obvious next step is to push further in the same direction (for example 48–64 × at ≤ 32_768) and find the point where stale counts start to block new hot keys. A separate, untested question is whether a longer period would also help at 131072.
