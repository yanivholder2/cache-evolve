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
