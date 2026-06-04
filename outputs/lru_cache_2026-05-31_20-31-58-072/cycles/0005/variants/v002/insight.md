# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Seeding freshest-quarter ghost-hits with `referenced: true` did not move hit-rate at all versus the parent's uniform `referenced: false` seeding, so the extra pass-2 protection earned no extra retained re-references on lirs:multi1.

## Takeaway

Differentiated promotion strength keyed on ghost-ring position is a no-op on this trace: freshest-quarter ghost-hits either get re-touched again before pass 2 ever reaches them (so the extra ref bit is redundant) or get displaced for reasons the bit cannot defend against, leaving the resident set bit-for-bit identical to parent v001. The signal is that ghost-ring *position* is not a usable freshness gradient on lirs:multi1 — once a key has been promoted Hot via ghost-hit, the surrounding CLOCK sweep already decides its fate, and adding one extra survival pass does not shift bucket-movement. Future gains should target the Hot-pool budget or the Cold→Hot rule itself rather than tuning per-ghost-hit promotion strength.
