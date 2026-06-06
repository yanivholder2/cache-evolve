# Cross-Cycle Insights

## Recurring wins

- Cheap sketch-efficiency levers that preserve admission semantics keep paying
  off: narrowing counters and swapping in a faster hasher have each won across
  both cycles, buying speed/memory for zero-to-marginal hit-rate cost. The
  CountMinSketch consistently carries slack that can be traded for cost.
- Changes that leave residency logic and the frequency signal untouched
  outperform changes that alter admission — the safest reliable gains come from
  reshaping *how* the sketch is computed/stored, not *what* it admits.

## Recurring failures

- Altering the frequency signal itself does not pay: frequency aging lost
  outright, apparently because the trace has stable, low true frequencies, so
  decay erases useful mass and counters never saturate.
- Single-lever tweaks off the shared baseline are a thinning vein: by Cycle 0002
  the easy isolated wins are largely harvested, and each remaining lever now
  moves one axis a little while nudging another the wrong way.

## Open questions

- Do the proven efficiency winners compose additively (u8 counters + fast
  hasher), or do their individual costs stack when combined in one variant?
- Can the persistent p99 tail regression be amortized away (e.g. precomputed
  estimates on the eviction path) rather than accepted as the price of each win?
- Is frequency aging a true dead end, or just mistuned — would a much wider
  decay interval with an amortized (non-inline) sweep change the verdict?
- How far can counter narrowing go (e.g. 4-bit packing) before the memory win is
  eaten by tail-latency cost?

## Per-policy notes

- Strict-LRU-head eviction is the untouched baseline residency policy; the
  efficiency levers all sit alongside it without modifying it.
- Sampled-coldest victim walk (vs strict LRU) does lift hit-rate, but only
  marginally and at the heaviest tail-latency cost of its cohort — treat as
  provisional; retune sample width or drop unless its lift can be bought without
  the eviction-path p99 hit.
- Tail latency (p99) is the recurring soft spot across *every* policy and lever
  touched so far — inline table sweeps, byte-narrow access, `as u32` mapping, and
  per-eviction sample walks all introduced p99 variance even when p50 improved.
