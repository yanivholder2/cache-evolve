# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Warm-starting ghost-readmitted entries to `freq = 1` gives every single-ghost-hit key an extra second-chance round in `evict_main()`, so they linger past one eviction pass and crowd out genuinely hotter main residents on this capacity-bound queue — net hit-rate fell below the parent.

## Takeaway

A lone ghost hit is too weak a reuse signal to deserve protection here: pre-loading `freq = 1` on re-admission keeps one-time-reuse keys resident at the expense of keys that earned frequency through real `get()` traffic. The parent's `freq = 0` re-admission — letting a ghost-readmitted key prove itself again before surviving an eviction pass — is the better default for this workload; the discriminating signal is accumulated access frequency, not the ghost hit itself.
