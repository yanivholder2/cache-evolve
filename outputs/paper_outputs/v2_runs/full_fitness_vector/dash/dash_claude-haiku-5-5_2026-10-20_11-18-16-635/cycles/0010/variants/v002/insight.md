# Variant v002 Insight

## Helped

## Hurt

- hit-rate: The hypothesis said the tag check would only change probe-time comparisons and leave eviction and promotion untouched, so hit rate should have matched the parent exactly; it fell instead, which points to the tag bookkeeping (set on insert, cleared on eviction and replacement) altering which entries survive or get found, or to tag mismatches turning real hits into misses.
- per-op-time: The get path got slower at both p50 and p99 even though the change was meant to skip full key compares; the keys here are u64, so a full compare is a single integer comparison and the per-probe tag read and branch add about as much work as they remove.
- per-op-time: Put latency also rose at both p50 and p99, which the hypothesis never predicted; the only put-side change is the extra tag set and clear on the insert and eviction paths, so that bookkeeping is the likely cause.

## Takeaway

For u64-keyed records, skipping the full key compare saves almost nothing, so a per-slot tag only pays off if key comparison is expensive; here it cost hit rate and latency on both get and put. The mechanism behind the hit-rate drop is inferred, not confirmed: the overlay source could not be inspected in this session, so whether the spare-bit assumption held or the tag was mis-maintained on promotion is still open and should be checked before any tag-based probe filter is retried.
