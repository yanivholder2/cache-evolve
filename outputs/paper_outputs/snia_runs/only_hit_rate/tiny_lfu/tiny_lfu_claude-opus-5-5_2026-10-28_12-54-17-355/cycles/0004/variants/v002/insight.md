# Variant v002 Insight

## Helped

(none: no trace/size cell improved over v001)

## Hurt

- hit-rate: hm_0 fell slightly at both sizes, with the larger drop at the small size, which is the opposite of what H02 predicted. Letting the candidate win ties at `cand_freq >= 2` means each tie now evicts the probation LRU, an entry already in main with an equal frequency estimate, and puts a key that has only just left the window at probation MRU. With a 20% probation segment, the extra admissions add churn and push out main entries that would have been re-referenced. Recency as a tie-breaker did not make up for this.
- hit-rate: web_0 also fell slightly at both sizes, so the `TIE_ADMIT_MIN_FREQ = 2` floor did not keep the scan resistance SLRU won. One likely reason: the sketch is incremented in both `get` and `put`. If a key misses and is then inserted with get-then-put, as the in-file workload test does, a key seen once already has an estimate of 2. It clears the floor, and a one-hit scan key can then win ties against cold probation entries. The new unit test does not catch this, because it builds its estimate-1 case with `put` only.

## Takeaway

Under v001's SLRU, resolving admission ties toward the window candidate hurts both traces, so strict `>` (victim wins ties) is the better rule here. That answers the open question: the tie-break still matters, but recency is the wrong way to break the tie. The floor of 2 is not a real one-hit filter, because one miss-then-insert adds two increments to the sketch. Any later admission rule keyed on a frequency threshold should first fix the sketch so it counts one access once, or set the threshold with the double count in mind. Otherwise it will not separate one-hit keys from re-referenced ones.
