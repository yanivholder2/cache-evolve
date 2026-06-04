# Variant v001 Insight

## Helped

*(no dimensions helped)*

## Hurt

*(no dimensions hurt)*

## Takeaway

Removing `timestamp: Instant` from `Item<K,V>` left `hit-rate` completely flat, confirming that `ClassicLRU` never touches the field and that `TimestampLRU`'s degraded FIFO-with-LRU-touch replacement is behaviourally inert on the workload-a benchmark. The expected payoff — a `metadata-size` dimension dropping from 40 to 24 bytes — never appeared in the fitness vector, exposing that the `evolve_metadata_size` binary is not yet wired into the evaluation pipeline. The structural improvement is real in the compiled artifact but invisible to the fitness system, so future variants targeting memory efficiency will remain unmeasured until the evaluator gap is closed.
