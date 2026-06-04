# Variant v001 Insight

## Helped

- hit-rate: When v003's leftmost-zero scan fails in the all-touched regime, halving every slot's `lfu_counter` in place drops the previously-lowest counter to 0 while preserving rank order between long-lived items, so the re-scan now evicts the least-frequently-touched slot instead of v003's blind slot-0 fallback.
- metadata-size: Aging reuses the existing `lfu_counter` field that v003 already carries, so the new logic adds zero bytes per `Item` or per `AssociativeCache`.

## Hurt

## Takeaway

Decaying counters only at the exact moment v003's untouched-first scan fails — not on every hit, as cycle/0003/v001 did — is what unlocks the all-touched regime without poisoning the re-reference signal. Inside a Dash bucket, an aging step is cheap only because it runs at eviction frequency rather than hit frequency, and a right-shift is the minimum-state operator that both manufactures a fresh zero-counter victim and keeps the relative ordering of survivors intact.
