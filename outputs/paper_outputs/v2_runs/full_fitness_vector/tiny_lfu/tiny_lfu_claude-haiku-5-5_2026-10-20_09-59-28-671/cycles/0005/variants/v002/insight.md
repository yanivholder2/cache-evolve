# Variant v002 Insight

## Helped

- metadata-size: Moving the per-row sketch seeds out of a per-instance field and into a module-level `const` table removes the per-cache seed storage that made `TinyLfuCache` grow in v001, so the struct returns to a size close to the parent's v002 baseline and under the hypothesis's target.
- per-op-time: With the seeds read from a `const` table instead of a per-instance field, both get and put got faster than v001, so the cached-seed speedup from v001 survived the layout change. The source could not be inspected in this session, so the mechanism is inferred from the hypothesis and the measured direction.

## Hurt

- (none)

## Takeaway

The change confirms that the v001 slowdown and the v001 size growth both came from storing seeds per instance, and that a `const` table keeps the speed without the struct cost. The hit rate did not move, which matches the claim that only storage layout changed. Whether the const-table indices really equal the per-access derivation is not verified here, since the overlay source and its equivalence test could not be read; the unchanged hit rate is consistent with that but does not prove it.
