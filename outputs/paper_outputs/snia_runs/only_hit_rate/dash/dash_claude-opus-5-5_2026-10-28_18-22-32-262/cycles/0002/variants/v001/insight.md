# Variant v001 Insight

## Helped

- hit-rate (16K, both traces): With the two-hit gate, an item hit only once in the stash moves to the stash MRU end and does not go into its target bucket. That item therefore no longer pushes a target resident down the `target -> stash` cascade. At the small size, each target slot is scarce, so avoiding these swaps gave a small gain on both `web_0` and `hm_0`, which is the effect the hypothesis credited to the gate.
- hit-rate (128K, `hm_0`): A small gain. The gate's filtering still helped slightly on a trace where the parent's MRU-end demotion was already strongest.

## Hurt

- hit-rate (128K, `web_0`): Here the gate is a net loss. At the large size, target buckets have spare room, so letting an item in after one hit costs little. Requiring a second hit keeps reused items in the stash, where they compete for space with new `put` arrivals. The gate also resets `lfu_counter` to 0 on every stash entry from a normal bucket. As a result, an item that was demoted after earning a target slot must be hit twice again before it can return. This partly cancels the protection that the inherited `STASH_DEMOTION_POS = Back` gives demoted items.

## Takeaway

The hypothesis was that the two mechanisms are independent: the gate guards stash exit, and MRU-end demotion guards stash entry. The results mostly don't support that. Combined, they gain only slightly over v001 alone at 16K, and they lose ground at 128K on `web_0`. The likely cause is that both mechanisms work through the same item, the demoted item in the stash. The counter reset on demotion means the gate penalizes exactly the items that `Back` placement is meant to favor. A natural next test is to keep the gate for fresh `put` arrivals only, so that items demoted from a normal bucket keep their count or promote on their first hit.
