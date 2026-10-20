# Variant v003 Insight

## Helped

- per-op-time: The get path is slower-to-faster on the measured run, but the hypothesis left `get` untouched, so this gain is not directly explained by the ghost table and is more likely noise or a secondary effect of different occupancy. Treat it as unconfirmed.

## Hurt

- hit-rate: Ghost-hit re-admission places evicted keys straight into the main table, which appears to displace resident entries on workload-a without a matching gain in re-referenced hits, so the net hit rate went down slightly.
- per-op-time: Every insert now checks the ghost ring and every `evict_one()` now writes a fingerprint into it, adding work to the put path that the baseline does not do.
- metadata-size: The ghost ring's cost is not visible in the reported metadata-size dimension, which still reads the baseline record sizes, so the ring's memory overhead is unmeasured by this variant's fitness vector.

## Takeaway

Direct re-admission of recently evicted keys into the main table did not pay for its put-path bookkeeping on workload-a: the ring adds per-insert and per-eviction work and the re-admitted keys seem to crowd out residents rather than add hits. Any future ghost-based variant needs to show a hit-rate gain first, and it must report the ring's bytes in metadata-size so the cost is visible in the fitness vector.
