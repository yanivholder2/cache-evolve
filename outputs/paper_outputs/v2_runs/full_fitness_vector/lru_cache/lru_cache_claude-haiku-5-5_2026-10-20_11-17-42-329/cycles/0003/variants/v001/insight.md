# Variant v001 Insight

## Helped

- hit-rate: Giving a referenced protected-segment head one rotation to the protected MRU end before demotion keeps frequently hit protected keys from losing protection on their first demotion, which recovers part of the loss the parent took from its single rescue.

## Hurt

- per-op-time: The extra unlink and push on the miss path, taken only when a referenced protected head is demoted, lengthens both `get` and `put` tail latency and p50, even though the hit path was meant to be untouched.

## Takeaway

A single bounded second chance on the protected side buys back a small amount of hit-rate at a visible per-op cost, and metadata size is unchanged because no node fields changed. The `get` movement is larger than the miss-path-only cost the hypothesis predicted, since `get` was not edited, so part of it is likely measurement noise or code-layout effect rather than the rotation itself; the next cycle should re-measure before crediting or blaming this change on the hit path.
