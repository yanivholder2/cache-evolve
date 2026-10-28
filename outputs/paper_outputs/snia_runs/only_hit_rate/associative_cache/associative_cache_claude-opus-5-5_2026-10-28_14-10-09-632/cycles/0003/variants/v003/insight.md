# Variant v003 Insight

## Helped

- hit-rate (`web_0` at 16384): In 127-slot buckets, the 64-slot floor moves the probation insertion point from index ~31 to index 63. Cold blocks get about twice as many misses to show a medium-distance reuse before reaching the eviction end. This brings back the web_0 small-bucket gain that v001's floor had shown and that v002's BIP sampling had not delivered on its own.

## Hurt

- hit-rate (`hm_0` at 16384): Deepening the insertion point shrinks the region above it, where promoted blocks and BIP's MRU samples live, from about three quarters of the bucket to about half. Items in that region age only from MRU inserts and promotions, not from ordinary probation inserts. A smaller region means hm_0's long-reuse blocks are pushed out sooner. That cancels the very gain v002's MRU sampling had produced, and hm_0 falls back toward v001's level.
- hit-rate (both traces at 131072): No real movement, as predicted. At 1023 slots, `insertion_index` returns `len/4` = 255 (larger than the 64-slot floor), so the code path is identical to v002. The tiny shifts are noise, not an effect of the change.

## Takeaway

The probation floor and BIP sampling are not additive. Both draw on the same fixed slots in a bucket: deeper probation insertion comes directly out of the protected region that keeps MRU-sampled and promoted blocks alive. At small buckets this variant trades hm_0's long-distance reuse for web_0's medium-distance reuse, which is the outcome H03 named in advance as showing the two mechanisms overlap. This suggests that the split between probation and protected space has to be chosen per workload, for example by set-dueling between the v001 and v002 insertion depths. A single combined depth cannot serve both traces.
