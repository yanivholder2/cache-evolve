# Variant v003 Insight

## Helped

## Hurt

## Takeaway

H03 added an admission filter at probing-tier exit: entries with zero probing hits are dropped instead of moved to the stash, to keep one-off items from displacing stash residents with proven reuse. On LIRS workload-a at size 2048, hit-rate was effectively flat against the parent, so the filter did not relieve stash pressure in a measurable way. The bypassed-entry count that H03 asked for is not in the eval artifacts, and the overlay source could not be inspected in this session, so this reading rests on the hypothesis and the fitness vectors alone.
