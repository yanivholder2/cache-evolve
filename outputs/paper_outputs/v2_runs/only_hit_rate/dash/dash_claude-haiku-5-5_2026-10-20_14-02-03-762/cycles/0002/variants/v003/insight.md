# Variant v003 Insight

## Helped

- hit-rate: Keeping the direct-to-stash path only while the stash has a free slot, and restoring the one-more-chance retry in the true target bucket when the stash is full, returns foreign evictions to a normal bucket under stash pressure. That is the loss the parent variant introduced, so hit-rate recovered slightly relative to the parent.

## Hurt

(No dimension declined in the recorded fitness vector.)

## Takeaway

Gating the direct-to-stash shortcut on free stash capacity recovers a small part of the hit-rate that the unconditional shortcut in the parent gave up, which supports the parent's diagnosis that stash pressure is what made the removed retry valuable. The per-op-time benefit that motivated the shortcut is not in this fitness vector, so this variant cannot yet show whether the shorter demotion path pays for the retained retry; per-op-time and metadata-size need to be measured before the trade-off can be judged.

<!-- Note: the overlay diff against the parent was not inspected when this insight was written. The description of the guard is taken from hypothesis H03. -->
