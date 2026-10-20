# Variant v001 Insight

## Helped

- per-op-time: The put path is faster, which matches the hypothesis's aim of hashing each key once per `put()` and reusing that hash for the admission check and the sketch increment instead of rehashing it in each call.
- per-op-time: The get path is also markedly faster even though the hypothesis only edits `put()` and the sketch helpers. The likely source is the shared sketch helpers that `get()` also calls, but the overlay source could not be inspected to confirm this, so treat the attribution as unverified.

## Hurt

- hit-rate: The hypothesis said admission order and eviction would be untouched, so hit rate should have stayed identical to the parent. It moved slightly down instead. Either the hashed sketch helpers are not exactly equivalent to the public methods they replace (for example, a different hash seed or index derivation), or the admission check now sees a different estimate. This is the claim the variant failed to uphold, and the source would be needed to pin down which.

## Takeaway

Hoisting the hash out of `put()` and sharing it with the sketch is a real constant-factor win on the put path, but it was not behaviour-preserving: the small hit-rate drop shows the hashed helper diverged from the original sketch lookup. The next check should be whether the precomputed-hash helpers produce the same counter indices as the public methods on the same keys.
