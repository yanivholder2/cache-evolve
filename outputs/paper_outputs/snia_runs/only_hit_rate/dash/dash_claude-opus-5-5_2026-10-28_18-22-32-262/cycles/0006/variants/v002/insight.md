# Variant v002 Insight

## Helped

- (none) No trace/size cell went up. The cell the hypothesis targeted, `web_0` at 16K, is the one that fell the most.

## Hurt

- hit-rate (16K, `web_0`): This is the largest drop, and it hit the very cell the change was meant to lift. A fingerprint now lasts four times as many evictions, so a lot of keys from long ago come back still holding ghost credit. Each one is promoted on its first stash hit and pushes out a resident that was more useful. So `web_0`'s returning keys at 16K were never just under the old ring's reach. Most of them return too late to be worth caching, and stretching the ring only credits more of them.
- hit-rate (16K, `hm_0`): A small loss. This is the stale-credit risk the rationale named. Every extra early promotion costs little, as v003 showed, but they add up to a slight net loss rather than holding level.
- hit-rate (128K, `web_0` and `hm_0`): Both cells slipped by a small amount. At 128K the stash is larger, so the factor-2 ring already covered the short reuse window that v003 had gained from. The extra reach adds only older, weaker evidence.
- The counting pre-filter does not change behaviour. It only lets `remove` skip the scan for absent fingerprints, and it never gives a false negative. So all the movement in these cells comes from the reach change alone.

## Takeaway

Making the ghost ring longer is not a lever that helps. On `web_0` at 16K, the extra evictions it remembers cost more than they return, because promotions on old credit push out better residents. So v003's short ring was not the 16K bottleneck. The factor of 2 was already close to the useful reach, and the flat `web_0` 16K cell has to be fixed somewhere other than ghost span: placement or demotion-side credit, not memory length.
