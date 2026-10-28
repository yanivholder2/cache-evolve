# Variant v001 Insight

## Helped

- hit-rate (web_0 @ 131072): The main-only veteran bit lets a 768-slot halving main keep a block that has been hit in main for one extra sweep visit after its counter decays to 0. A fresh promotion, whose counter is reset to 0 and whose recent bit is cleared, now loses that tie. This cell clears the refutation bar, and the gain matches the one the hypothesis rationale cites for cycle 0009 v003's veteran bit alone. That fits the parent's finding that linear decay never touches 1024-slot buckets: in this cell the veteran bit is the only active change, so the two changes stack additively.
- hit-rate (hm_0 @ 131072): This cell rose by the same mechanism and also clears the refutation floor. Again the gain matches the cited v003 delta, so halving-mode buckets with v001's CRP filter respond to the veteran step exactly as v003's buckets did.

## Hurt

- hit-rate (hm_0 @ 16384): The cell dipped and landed exactly on the hypothesis's tolerance bound, so the side-claim survives only at the limit. In a 64-slot linear-decay main, `steps` is already `recent + counter`, with eight fine tiers. Adding a +1 veteran step to every main-hit slot pulls many once-hit veterans level with the next counter tier. That blurs the fine tiers that gave the parent its small-bucket gain. Because almost every resident main block has been hit in main, the bit also tells residents apart less well in a small main than in a large one.
- (web_0 @ 16384 did not move. The veteran step there seems to be absorbed by the linear tiers without changing the eviction order in practice.)

## Takeaway

Linear decay and the veteran bit act on disjoint bucket sizes. Linear decay pays off only in 64-slot mains and the veteran bit only in 768-slot halving mains, so combining them reproduced each one's gain at its own size. The veteran bit gave nothing back in small buckets and cost a little on hm_0, as the hypothesis's refutation clause anticipated. That supports gating `veteran` (both the set in `earns_veteran` and the `steps`/`sweep_visit` term) to `self.halving` buckets. The gate should keep the large-size gain and restore the parent's small-bucket behavior exactly.
