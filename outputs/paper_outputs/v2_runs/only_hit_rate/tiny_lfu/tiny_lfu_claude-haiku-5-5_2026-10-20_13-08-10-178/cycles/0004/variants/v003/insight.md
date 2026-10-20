# Variant v003 Insight

## Helped

## Hurt

- hit-rate: The change swapped the demotion call in `touch()` from `push_back` to a new `push_front` helper, so a protected entry demoted from the protected segment now lands at the opposite end of probation from v002, which lowered hit-rate on workload-a at size 2048 relative to the parent.

## Takeaway

Moving demoted protected entries to the other end of probation was worse than v002's placement on workload-a, so the second pass v002 gave demoted entries was worth keeping. The hypothesis describes the two ends of probation inconsistently (push_front is called the "next victim candidate," while the rationale calls v002's placement MRU), so the exact mechanism is unconfirmed; the result is a single trace at one cache size and should not be taken as a general ruling on demotion placement.
