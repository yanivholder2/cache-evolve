# Variant v003 Insight

## Helped

- hit-rate: Sampling up to 5 nodes forward from the LRU head and evicting the
  lowest-`sketch.estimate` candidate replaces a genuinely cold item instead of
  a recently-touched-but-rarely-used head node, so more hot keys survive.

## Hurt

- (none): No measured dimension regressed; the change overwrites a single
  resident slot in place, so capacity and value handling are untouched.

## Takeaway

Widening victim selection from the strict LRU head to a small frequency-ranked
sample yields a real but slim hit-rate gain on workload-a — the LRU tail rarely
hides a much-colder item than the head, so the upside of looking deeper is
small at SAMPLE=5. A larger sample or a workload with more tail churn would be
needed to see whether this effect grows.
