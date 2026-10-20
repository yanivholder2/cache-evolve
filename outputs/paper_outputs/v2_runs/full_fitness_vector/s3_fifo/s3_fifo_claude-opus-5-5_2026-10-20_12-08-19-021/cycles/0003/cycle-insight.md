# Cycle 0003 Insight

## Cohort outcome

All three hypotheses used cycle 0002's hashed v003 as their parent, and each made a different kind of change:

- **H01** replaced the lopsided adaptation with ARC-style two-ghost signals.
- **H02** replaced the key-cloning ghost with a fixed ring of fingerprints.
- **H03** replaced the whole adaptive policy with the paper's static 10/90 split and its `freq >= 1` graduation rule.

All three are Winners. Only v002 (H02) improved on its parent on both hit-rate and latency. v001 and v003 each lost a little hit-rate to the parent and made it into the Winners on relative rank in the cohort.

## Patterns

- **The incumbent policy has now held up three cycles in a row.** v003's lopsided adaptation with two-touch graduation has outlasted correctly directed ARC signals (v001) and the published static setup (v003). So the adaptive gain is not just an escape from a bad 25/75 start. Something in the "adaptation + `freq > 1` graduation" package is earning a small but real margin.
- **Both losing variants changed two levers at once.** v001 changed the signal direction and also let main-ghost hits bypass `small`. v003 changed the split and the graduation rule together. In neither case can we tell which lever cost the hit-rate.
- **The ghost behaves like a policy knob, not plumbing.** v002 was meant to be policy-neutral. But its ring shortens the ghost's effective memory: holes left by taken entries still use up positions. That made ghost hits rarer, and the split drifted in a favourable direction. In the other direction, v001's extra ghost traffic pushed the split around in a way that hurt. Ghost size and contents feed the adaptation directly.
- **Most `put` latency gains are real; `get` gains may be noise.** `put` gains trace to specific removed work: the clones and `retain` compaction in v002, and the resize branches in v003. But v001 and v003 also report `get` gains with `get` code byte-identical to the parent. That points to benchmark variance or host drift, which also casts doubt on part of v002's `get` gain.
- **`metadata-size` still misses all ghost cost.** It hid v001's second ghost, v003's larger ghost, and v002's real heap savings.

## Open questions

- **Which half of each two-lever change caused its loss?**
  - For H03: fixed 10/90 split with the parent's `freq > 1` graduation (no freq reset).
  - For H01: keep the main ghost but reinsert its hits into `small`, or restore v003's small-ghost direction.
- **Where does `small_size` actually settle?** No variant has reported it yet, even though cycle 0002 asked for it. Does the ring ghost move it?
- **Is ghost size or hole handling a tunable lever for hit-rate on the ring base?**
- **Are the `get` latency gains real?** This needs a same-host re-baseline of the parent.

## Continuation

Adopt v002 (the fingerprint ring) as the new parent. It wins on both axes and keeps the incumbent policy. Drop wholesale policy replacements and ARC-direction rewrites. Instead, test one lever per variant: graduation rule alone, fixed split alone, ghost capacity or hole handling alone, and main-ghost reinsertion into `small`. Every variant must report where `small_size` settles. Credit `get` latency changes only when the `get` path itself changed.
