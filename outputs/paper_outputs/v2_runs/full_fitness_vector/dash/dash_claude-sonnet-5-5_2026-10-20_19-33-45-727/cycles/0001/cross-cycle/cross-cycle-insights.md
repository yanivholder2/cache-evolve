## Recurring wins

- Scan resistance is the shared theme so far: uniform MRU insert, cold-first tiered insert and a second-hit admission gate all won modestly on the recency-skewed LIRS trace.
- Every win to date is small. Insert-position and admission tweaks each help, but neither looks like a main driver.
- Position-only changes cost no metadata, so they are a safe baseline for gains.

## Recurring failures

- Adding a per-item admission flag grew the item struct (no spare padding), so it paid a metadata cost. Avoid flag designs that need new bytes.
- Single-position-only variants produce mixed latency shifts that are plausibly noise, since no work was added or removed. They give little new information unless they come with a latency re-measurement.

## Open questions

- Is the latency change real or noise? Position-only variants changed no work, so re-measure before trusting it.
- Does the uniform-MRU gain come from stash→target promotion or from demotion cascades? The two were changed together.
- Can the second-hit filter work with no extra footprint, for example by packing the flag into spare bits or encoding it in bucket position?
- Do the position and admission gains stack?
- Do these gains hold on traces that are not recency-skewed?

## Per-policy notes

- DASH (stash promotion): the second-hit gate has the only mechanically explained speedup, since it skips remove-and-promote work on a first hit. Position-only variants have no such explanation.
- Next steps: combine the footprint-free second-hit filter with the tiered insert positions, and separate the promotion and demotion position choices to attribute the gain.
