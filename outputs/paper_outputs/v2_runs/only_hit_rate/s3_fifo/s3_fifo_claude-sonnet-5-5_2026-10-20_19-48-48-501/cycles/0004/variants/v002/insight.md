# Variant v002 Insight

## Helped

- None. hit-rate did not move relative to the parent.

## Hurt

- None. hit-rate matches the parent, so the stricter gate cost nothing measurable either.

## Takeaway

Tightening the small-to-main promotion gate by one frequency level (H02) produced a hit-rate identical to the parent's on this trace and cache size. Either the change did not alter which keys get promoted (for example the gate was already effectively met or unreachable, or the change was not exercised), or the promotion threshold is not a binding lever here. Treat "stricter admission always wins" as unsupported for the promotion path. I could not inspect the overlay sources with the tools available, so I can't say which of these explanations holds. Confirm the diff actually changed behaviour before spending more cycles on this lever.
