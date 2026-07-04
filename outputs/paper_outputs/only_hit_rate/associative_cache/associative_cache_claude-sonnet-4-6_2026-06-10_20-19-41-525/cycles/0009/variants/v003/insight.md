# Variant v003 Insight

## Helped

- hit-rate: The three-way combination — SLRU tier separation, 4x-wider buckets, and cold-first/decay-by-2 eviction confined to probationary — produced a substantial lift over baseline ClassicLRU: wider buckets increase the pool of candidates per hash slot, SLRU's probationary/protected boundary prevents one-hit wonders from ever reaching the protected segment (a stronger guarantee than insertion-position heuristics alone), and cold-first scan within probationary accelerates removal of genuinely dead items so reuse-worthy items are not delayed behind cold chaff.

## Hurt

_(no dimensions regressed in the evaluated fitness vector)_

## Takeaway

The key lesson from v003 is that wider geometry and SLRU tier separation are mutually reinforcing: the 4x-wider buckets give the probationary filter a larger, richer population to work with, so its sorting between one-time and recurring items is more reliable and the protected segment fills with genuinely hot items rather than accidental early-arrivals. Confining decay to the probationary segment only — leaving protected-item counters untouched during warm-lock events — turned out to be critical: it means promotion into the protected tier is stable and cannot be eroded by the high-churn activity in probationary, preserving the working-set capture benefit that makes SLRU worthwhile in the first place.
