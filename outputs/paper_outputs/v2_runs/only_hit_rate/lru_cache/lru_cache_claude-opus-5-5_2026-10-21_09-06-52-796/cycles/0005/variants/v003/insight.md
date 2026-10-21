# Variant v003 Insight

## Helped

- hit-rate: A 4-row TinyLFU sketch now guards both routes into `main`: the ghost-hit insert in `put` and the `freq > 0` promotion out of `small` in `evict_one`. It compares each candidate against the entry that `main_victim()`'s CLOCK sweep would actually evict. Admission only happens when the candidate's long-run count is strictly higher. A key that came back once within the ghost window, or was touched once while in `small`, can no longer push out a `main` resident that the sketch has seen more often. The gain is marginal, though: the gate is idle until `main` fills its `capacity - small_target` share, and it only rules on candidates that the parent's recency signals had already passed.

## Hurt

(No dimension regressed. The fitness vector reports only `hit-rate`. The change has real costs that went unmeasured, so they are untested rather than free. For `metadata-size`, the sketch is `4 * next_pow2(2 * capacity)` bytes, about 16 KiB at size 2048. That is extra memory on top of the ghost structures the parent had just shrunk. For `per-op-time`, every `get` and `put` now pays for an extra `DefaultHasher` pass and 4 counter updates, hit or miss. Each time the gate is active it also hashes the victim's key again and may run the CLOCK sweep. Periodic halving walks the whole table every `10 * capacity` accesses.)

## Takeaway

Gating every route into `main` against the true CLOCK victim still bought only a hair of hit rate on workload-a at size 2048. In this S3-FIFO, the small-queue filter plus a `capacity / 4` ghost already turns away nearly all the low-frequency candidates that TinyLFU exists to block. Fixing the leak and the wrong comparison target did not unlock a bigger frequency-admission gain. Keep the sketch only if `per-op-time` and `metadata-size` show its per-access hashing and table cost are close to nothing. Otherwise this result says frequency admission is near its ceiling on top of this design, and effort should go elsewhere.
