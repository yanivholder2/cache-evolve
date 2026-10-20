# Variant v001 Insight

## Helped

## Hurt

- per-op-time: Get latency regressed on both median and tail, and put latency regressed too, even though the hypothesis only reordered lookups in `get` and left `put` untouched. Reordering target-bucket and probe checks ahead of the stash scan should not slow target hits, so the slowdown is more likely from run-to-run variance or an unintended side effect in the overlay than from the lookup order itself.

## Takeaway

Moving the stash scan behind the target and probe checks did not produce the get-latency win that cycle 0008 v003 showed on its earlier base; on the packed-counter parent, get got slower rather than faster. Hit rate and metadata size held exactly, which confirms the reorder preserved semantics, but the fitness vector does not split get times by target hit versus stash hit, so the stash-hit tail the hypothesis flagged as the risk remains unmeasured. The overlay source could not be inspected in this session, so the cause of the put regression is unexplained and should be checked before this reordering is carried forward.
