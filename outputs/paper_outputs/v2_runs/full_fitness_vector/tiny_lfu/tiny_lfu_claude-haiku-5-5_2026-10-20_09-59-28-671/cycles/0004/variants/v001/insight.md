# Variant v001 Insight

## Helped

- per-op-time: Get and put both got faster than the parent v002, which is the direction H01 predicted: per-row sketch seeds are computed once at construction and read from storage, instead of being re-derived on every sketch access. The tail latency for put improved the most, consistent with removing per-access seed work from the hot path. The overlay source could not be inspected in this session, so the link to the seed caching is inferred from the hypothesis and the measured direction, not confirmed in code.

## Hurt

- metadata-size: The TinyLfuCache struct grew noticeably relative to the parent, even though H01 asked that the struct growth stay limited to the seed storage and that a `const` table be preferred if the size grew. The growth points to the seeds being stored as a per-instance field rather than a `const` item, which is the variant the hypothesis said to avoid. The sketch's own counter storage is unchanged.

## Takeaway

Caching the per-row seeds at construction does buy back the constant-factor cost that v002 introduced, and it does so without any hit-rate change, which fits the hypothesis's claim that the counter indices are unchanged. The trade is the per-instance seed array, which adds fixed metadata on each cache; moving the seeds into a `const` table, as the hypothesis suggested, would keep the speed gain while holding metadata at the parent's size. Whether the indices are truly identical still needs a check of the overlay source, since the hit-rate match is consistent with that but does not prove it.
