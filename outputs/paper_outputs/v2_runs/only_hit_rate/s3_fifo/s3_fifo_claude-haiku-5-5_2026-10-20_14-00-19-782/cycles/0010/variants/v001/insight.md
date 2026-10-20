# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Moving main-evicted zero-frequency keys out of the shared ghost queue into a separate `main_ghost` ring capped at capacity/4 reduced the ghost-hit re-entries that the parent's shared-queue push provided, so re-references to those keys fall back to the small-queue admission path.

## Takeaway

Diverting main-evicted keys into a quarter-capacity side ring makes the parent's gain disappear, which means the shared ghost window was useful for these entries and the cap is too tight to hold them. The hypothesis assumed main-fed entries were crowding out small-queue ghost history; this result does not support that, at least on lirs workload-a at capacity 2048. Confidence is limited to one trace and one size, and the overlay source could not be inspected in this session, so the mechanism is inferred from the diff description and the parent insight rather than confirmed in code.
