# Variant v003 Insight

## Helped

## Hurt

## Takeaway

Adding a hard admission floor of `MIN_ADMISSION_FREQ = 2` — blocking any item with `incoming_freq < 2` from displacing a probationary victim — produced no measurable change in hit-rate on workload-a. The existing strict `>` comparison already prevents a first-encounter item (freq = 1) from displacing any resident with freq ≥ 1; the only additional scenario this floor blocks is an incoming item at freq = 1 beating a victim at freq = 0, which occurs only during the cold-start phase of a warm trace and is averaged away over the full run. On a stable Zipfian workload where the cache has long since warmed up, the early-trace edge case the floor targets is too rare to shift the aggregate hit-rate, confirming that the admission gate's existing strict inequality already captures most of TinyLFU's intended semantic on this workload.
