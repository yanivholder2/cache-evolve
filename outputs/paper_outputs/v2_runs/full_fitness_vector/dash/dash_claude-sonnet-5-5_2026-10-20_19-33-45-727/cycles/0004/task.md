# Task: cycle-insight-writer

Cycle: 0004
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0004

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0004/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0004/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0004/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0004/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0004/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0004/hypotheses/H03.md

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


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Three variants followed up on cycle 0001: one packed the second-hit flag into spare bits, one made first stash admissions cold on top of uniform MRU, and one stacked cold demotions on the second-hit filter. All three won. Only the bit-packing variant (v001) gained cleanly. The two position-based stacking attempts gave weak or mixed results.

## Patterns

- Bit-packing the admission flag (v001) was the clear win. It kept the filter's hit-rate benefit, restored the baseline item size, and improved latency. The flag does not need its own field.
- Position and admission levers overlap rather than add. Cold first admission on top of uniform MRU (v002) gave a small hit-rate gain at a latency cost. Cold demotions plus a flag reset on top of the second-hit filter (v003) gave faster operations but slightly lower hit-rate. Each pair targets the same thing, keeping weakly reused items out of the hot region.
- Hit-rate and latency traded off in the position variants. Whichever of the two improved, the other tended to give ground. The latency changes in v002 and v003 are explained only by inference, because the overlay source was not read.
- Insert position alone looks close to exhausted as a lever.

## Open questions

- Is v002's small hit-rate gain real, or within trace noise?
- Does v001's bit-packed filter keep its latency gain when it is combined with a milder form of demotion penalty, such as cold demotion without the flag reset?
- Does the packed flag have enough spare bits to hold a small counter, such as a 2-bit or 3-bit hit count, instead of a single bit?

## Continuation

Treat v001 (bit-packed second-hit filter, baseline item size) as the base for the next cycle. Explore richer admission signals that cost no metadata, such as a multi-bit hit counter or an age or recency bit in the same spare bits, instead of more insert-position tweaks. Drop further stacking of cold-demotion rules on the filter, and drop cold first admission, unless a latency re-measurement shows the effect is real.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

Three variants built on the bit-packed second-hit filter: a 2-bit hit counter in place of the 1-bit flag, a skip-if-already-hot shortcut on the get-hit path, and a CLOCK-style second-chance ref bit for victim selection. All were Winners, but only by negligible hit-rate gains or none. All three gave up latency.

## Patterns

- Metadata stayed flat everywhere. Spare bits can hold a counter or a ref bit, so footprint is no longer the constraint.
- Richer admission or recency signals (counter, ref bit) added almost no hit-rate. The second-hit filter already screens out most one-touch items, so there is little left for them to catch.
- Every variant added hot-path work (per-hit read-modify-write, a conditional guard, or an eviction scan) and every one lost latency. The shortcut meant to remove work also cost latency, because a branch that only sometimes succeeds is not free.
- Hit-rate gains here are within plausible trace noise. The Winner status says little about real improvement.
- Admission and eviction-order levers now look saturated on this trace, after the position levers in cycles 0001 and 0002.

## Open questions

- Is the remaining latency regression real or run-to-run noise? Several explanations rest on inference, because the overlay source was not readable.
- If the hit path is already cheap, where does get and put time actually go (lookup, locking, bucket scan)?
- Does a different trace, such as one with scans or a frequency skew, expose gains these levers cannot show on the LIRS trace?

## Continuation

Treat cycle 0002's v001 (bit-packed second-hit filter, baseline item size) as the base. Drop further admission-signal, recency-bit and hit-path-shortcut variants, because hit-rate is saturated and each costs latency. Aim the next cycle at latency work on the lookup, locking and bucket-scan paths, or at evaluating on a different trace. Repeat the latency measurement before trusting any result.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
