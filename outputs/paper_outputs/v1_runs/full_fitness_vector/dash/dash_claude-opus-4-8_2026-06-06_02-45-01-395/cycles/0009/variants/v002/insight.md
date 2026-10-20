# Variant v002 Insight

## Helped

- hit-rate: Deferring the index computations is a pure code-motion change with byte-identical branch bodies, so resident behaviour is untouched and hit-rate is preserved exactly.
- metadata-size: No struct or field changed — only statement placement inside `promotion_get` moved — so both `Item` and `Dash` footprints are unchanged.

## Hurt

- per-op-time: get-p50 rose rather than fell, directly refuting the hypothesis — removing the two `target_idx`/`probing_idx` `get_index` calls from the stash-resolving median path did not buy median get latency, and get-p99 ticked up too, suggesting the moved computation perturbed codegen/branch layout more than the saved work helped.

## Takeaway

Deferring `target_idx`/`probing_idx` past the stash branch confirms the median get is not bounded by those two `get_index` calls: they sat below the noise floor, and relocating them slightly worsened both get-p50 and get-p99 instead of improving them. The median stash-resolving get is hasher/probe-bound, not arithmetic-bound, so shaving "wasted" index math is not a viable lever here.
