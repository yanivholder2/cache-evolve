# Variant v001 Insight

## Helped

- hit-rate (`hm_0` at 131072): The only cell that improved, and only slightly. This fits PSEL staying mostly at or above the threshold in 1024-slot buckets. Those buckets then sample at `PERIOD_BIP = 16`, twice v002's rate, which gives more long-reuse `hm_0` blocks a full bucket lifetime. The gain is too small to separate from noise.

## Hurt

- hit-rate (`web_0` at 131072): This is the loss the hypothesis meant to remove, and it got worse instead of better. In a 1024-slot bucket, an MRU-inserted item lives roughly a full bucket's worth of insertions before eviction. The 8-entry `sampled` ring covers only 8 × 16 = 128 insertions at the BIP period. So most samples are overwritten before they die, and the `saturating_sub` penalty almost never fires. Meanwhile, short-reuse hits on fresh samples still give rewards. PSEL is therefore biased toward BIP, and one-touch `web_0` blocks are sent to the MRU tail more often than in v002.
- hit-rate (`hm_0` at 16384): The gain from v002 shrank a little. With 128-slot buckets, the ring covers about one bucket lifetime, so the feedback can reach LIP. But `PSEL_INIT == PSEL_THRESHOLD`, so a single unreused-sample eviction flips the bucket to `PERIOD_LIP = 128`. That sends noisy buckets to a much lower MRU rate than v002's fixed 32, which costs some of the long-reuse samples that made v002's gain.
- hit-rate (`web_0` at 16384): Essentially flat versus v002. The adaptive switch gave no clear win on the scan-heavy trace at the size where the feedback window is roughly sized right.

## Takeaway

A PSEL that scores MRU sampling by what happens to the sampled items only works if the sample tracker outlives the items. Here the 8-slot ring is about 8× too short for 1024-slot buckets, so evictions, the negative signal, are mostly lost. The counter then drifts toward the more aggressive BIP period exactly where `web_0` pollution hurt most. The reward is also miscalibrated: it credits any re-hit, including short-reuse hits that 25%-point insertion would have caught anyway, rather than the marginal benefit of MRU placement. To fix this, size the tracker to the bucket, or use a per-item "sampled" bit or a set-dueling leader/follower split. Credit only hits that land after the item has passed the probation window.
