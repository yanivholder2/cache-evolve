# Task: cycle-insight-writer

Cycle: 0002
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0002

## Winners

- v002
- v001
- v003

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0002/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0002/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0002/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0002/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0002/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0002/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three variants probed promotion policy in the DASH segment: two moved only the insert position (uniform MRU, and split cold/hot by tier), and one changed admission by requiring a second stash hit. All three won, each with a modest hit-rate gain on the recency-skewed LIRS trace.

## Patterns

- Every gain was small. Insert position and admission tweaks both help, but none looks like a main driver.
- The position-only variants cost no metadata. The admission-filter variant added a per-item flag that grew the item struct, because there was no spare padding.
- Latency shifts were mixed. They are plausibly noise in the position-only variants, where no work was added or removed. The only mechanically explained speedup came from the admission filter, which skips the remove-and-promote work on a first hit.
- Scan resistance (keeping one-hit-wonders out of the hot tiers) was the shared theme. Uniform MRU (v001), cold first admission (v002) and a second-hit gate (v003) all pursue it in different ways.

## Open questions

- Is the latency change real or noise? v001 and v002 changed no work, so re-measure before trusting it.
- Does the gain from v001's uniform MRU come from stash→target promotion or from the demotion cascades? The two were changed together.
- Can the second-hit filter work without growing the item, for example by packing the flag into spare bits or encoding it in bucket position?
- Do the position and admission gains stack?

## Continuation

Combine the second-hit admission filter (v003) with the tiered insert positions (v002 or v001), and make the filter's flag footprint-free so metadata stays flat. Separate the promotion and demotion position choices to attribute v001's gain. Drop further single-position-only variants unless they are paired with a latency re-measurement.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
