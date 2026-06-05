# Variant v002 Insight

## Helped

- hit-rate: Giving a stash overflow victim one cuckoo-style hop into a salt-derived alternate bucket (`stash_alt_index`) keeps cold keys resident a little longer, so a few more catch their second reference instead of being dropped — a small but positive move over baseline.

## Hurt

- (none): No measured dimension regressed; total stash capacity is conserved (the hop changes *which* victim drops, not how many slots exist), and the dual-home get path is gated so it collapses to baseline behaviour when `stash_size < 2`.

## Takeaway

A second legal stash home, reached by one relocation hop on overflow, lengthens cold-key dwell time along an axis untouched by the failed admission/capacity levers — and it does nudge hit-rate up. But the gain is marginal: most overflow victims still land in an already-full alternate and drop anyway, so a single hop extracts only a sliver of the dwell-time headroom the stash has.
