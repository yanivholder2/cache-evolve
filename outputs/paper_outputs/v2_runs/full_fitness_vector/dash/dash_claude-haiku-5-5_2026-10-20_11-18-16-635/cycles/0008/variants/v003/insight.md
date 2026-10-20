# Variant v003 Insight

## Helped

- per-op-time: Scanning the target bucket and its probe sequence before the stash in `get` removes the stash scan from the common target-hit path, which is where the get latency drop comes from.

## Hurt

- per-op-time: The `put` latency moved in the wrong direction at p50 even though this edit leaves `put`, promotion and `evict_one()` untouched, so that movement is not explained by the code change and is most likely measurement noise.

## Takeaway

Reordering the tier lookup in `get` to check the target and probe buckets before the stash cut get latency without changing hit rate or metadata size, which supports the cycle 0001 finding that the stash scan sits on the common get path. The stash-hit tail that the hypothesis flagged as a risk is not split out in this fitness vector, so the net effect on stash-hit gets is still unverified.
