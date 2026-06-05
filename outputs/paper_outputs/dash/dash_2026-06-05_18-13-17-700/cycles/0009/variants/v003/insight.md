# Variant v003 Insight

## Helped

- hit-rate: Adding a second probing home — lifting a target→probing overflow victim into its own probing bucket at `(target_idx + 2) % seg` (via the `probing_alt == victim_probing` rescue in `promote_to_probing`) instead of demoting it to stash — produced a small but real gain, because the victims spared here are proven-warm residents that earned their way up, not the cold overflow the stash breadth was rescuing.

## Hurt

- (none) — no dimension regressed; the rescue hop never evicts (`!is_full` guard) and only fires on the slot-legal own-probing case, so occupancy and `single_eviction_per_put` held without cost.

## Takeaway

The probing tier is the mirror image of the stash the parent tried: its overflow victims are hot keys, so a breadth-of-homes lever there converts to retained hits where the identical lever on the cold-stash victims (cycle 0008) was inert. The gain is only marginal because the rescue fires under a narrow condition — the victim's own probing bucket must equal `target+2` and have room — so it touches a thin slice of promotions, but it confirms the hypothesis that *which* keys a relocation spares matters more than the relocation breadth itself.
