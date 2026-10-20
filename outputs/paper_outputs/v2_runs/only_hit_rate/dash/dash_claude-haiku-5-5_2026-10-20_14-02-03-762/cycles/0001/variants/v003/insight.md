# Variant v003 Insight

## Helped

(No dimension improved in the recorded fitness vector.)

## Hurt

- hit-rate: Foreign residents evicted from a normal bucket now go straight to the stash instead of getting one retry in their true target, so items that previously stayed in a normal bucket now depend on the stash for residency and can be lost to stash pressure.

## Takeaway

The change removes the one-more-chance retry in `demote_from_normal_bucket`, so a foreign probing eviction is placed directly into `Slot::Stash` with `insert_pos` honoured. That shortens the demotion path, which is what the hypothesis targeted, but this variant's fitness vector records only hit-rate, so the per-op-time gain it predicted is untested here. The one measured dimension moved slightly down, which fits the loss of the target retry; a follow-up needs per-op-time and metadata-size measured to tell whether the shorter path pays for that hit-rate cost.
