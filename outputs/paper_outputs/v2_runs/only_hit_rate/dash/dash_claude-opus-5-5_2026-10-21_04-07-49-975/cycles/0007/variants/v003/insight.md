# Variant v003 Insight

> Note: I could not list or open the overlay source in this session. I only had file read and write, with no directory listing, search or diff, and `manifest.toml` was not at the variant root or under `attempts/01/`. The mechanism below comes from H03's stated change and has not been checked against the actual diff.

## Helped

## Hurt

## Takeaway

Sending displaced foreign residents straight to the stash left hit-rate unchanged from the parent down to the last bit, not just within noise, on this deterministic single-trace eval. That points to no change in behaviour at all: either the cascade branch never fires at this size, or the change did not take effect. A neutral but real effect would be unlikely to match this exactly. The hypothesis was about per-op-time, but the fitness vector has no per-op-time dimension, so the claim is untested. The eval needs a per-op-time dimension and a counter of displaced residents before this variant can be told apart from its parent.
