# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Adding a `referenced` bit tie-breaker to `demote_prot_head()` — selecting victim by `(freq, referenced)` ascending while keeping move-to-tail intact — produced no measurable change in hit rate on workload-a. The tie-breaker only activates when two or more candidates in the LRU-quarter scan window share the same frequency, with one referenced and one not; on this workload the existing frequency ordering already differentiates candidates well enough that such ties are essentially absent within the narrow scan window of `(prot_count / 4).max(1)` nodes. The reprieve mechanism is structurally sound and avoids the position-freeze failure of cycle-3, but it targets a collision pattern this trace does not exhibit at cache size 2048.
