# Variant v003 Insight

## Helped

- hit-rate: Adding `data.lfu_counter += 1` before `promote_to_target` in the stash-hit branch means items enter the target tier already protected under `ClassicLRUFreqProtect`, eliminating the one-access window where a genuinely accessed item was as eviction-vulnerable as a freshly inserted cold one.

## Hurt

## Takeaway

Crediting the stash-hit access on the `lfu_counter` before target insertion closed a real protection gap: stash-demoted items have their counter reset to zero, so without this change a stash-hit item would arrive in the target tier indistinguishable from cold newcomers and subject to immediate eviction. The improvement is positive but modest, indicating that the stash-exit vulnerability window was a genuine loss site at this workload yet not the dominant one — the larger gains seen from geometry changes in the parent suggest hot-tier capacity remains the bigger lever than per-item protection accounting.
