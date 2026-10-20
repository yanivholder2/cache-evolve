# Variant v002 Insight

## Helped

## Hurt

- hit-rate: The ghost list promotes any evicted-then-re-requested key *straight into the protected segment* via `push_back_protected`, so a single reuse across one eviction boundary is now treated as strong as v001's two-resident-accesses gate; this over-trust floods protected with marginal keys and, through `enforce_protected_budget`, demotes genuinely hot residents into probationary where they get evicted, lowering hit-rate.

## Takeaway

Extending reuse memory across one eviction is not free additivity on this trace: a short reuse distance is weaker evidence than a second *resident* access, and routing ghost hits directly to protected (rather than back to probationary) let one-shot post-eviction reuses crowd out the truly hot keys SLRU had earned. The ghost recognition was correct in mechanism but too aggressive in placement — promotion straight to protected regressed the only thing it aimed to lift.
