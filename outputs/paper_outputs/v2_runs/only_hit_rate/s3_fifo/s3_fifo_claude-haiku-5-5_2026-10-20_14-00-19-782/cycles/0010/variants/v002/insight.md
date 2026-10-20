# Variant v002 Insight

## Helped

## Hurt

## Takeaway

This variant is a pure refactor of the ghost queue (linear `Vec` scans replaced by a `VecDeque` plus a key-to-sequence index), and it was meant to cut miss-path cost while leaving eviction behavior unchanged. The only measured dimension, hit-rate, is flat against the parent, which is what the hypothesis predicted. However, the claimed benefit (per-op-time on the miss path) was not measured, and the evaluation shows no metadata-size data either, so the variant's speed effect is still unknown. Unverified: I could not read the overlay source in this session, so I did not confirm the edit is present in the built code as the hypothesis asks.
