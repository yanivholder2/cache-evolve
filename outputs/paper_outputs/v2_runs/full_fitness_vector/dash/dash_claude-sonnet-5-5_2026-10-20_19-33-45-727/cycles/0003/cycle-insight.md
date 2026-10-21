# Cycle 0003 Insight

## Cohort outcome

Three variants built on the bit-packed second-hit filter: a 2-bit hit counter in place of the 1-bit flag, a skip-if-already-hot shortcut on the get-hit path, and a CLOCK-style second-chance ref bit for victim selection. All were Winners, but only by negligible hit-rate gains or none. All three gave up latency.

## Patterns

- Metadata stayed flat everywhere. Spare bits can hold a counter or a ref bit, so footprint is no longer the constraint.
- Richer admission or recency signals (counter, ref bit) added almost no hit-rate. The second-hit filter already screens out most one-touch items, so there is little left for them to catch.
- Every variant added hot-path work (per-hit read-modify-write, a conditional guard, or an eviction scan) and every one lost latency. The shortcut meant to remove work also cost latency, because a branch that only sometimes succeeds is not free.
- Hit-rate gains here are within plausible trace noise. The Winner status says little about real improvement.
- Admission and eviction-order levers now look saturated on this trace, after the position levers in cycles 0001 and 0002.

## Open questions

- Is the remaining latency regression real or run-to-run noise? Several explanations rest on inference, because the overlay source was not readable.
- If the hit path is already cheap, where does get and put time actually go (lookup, locking, bucket scan)?
- Does a different trace, such as one with scans or a frequency skew, expose gains these levers cannot show on the LIRS trace?

## Continuation

Treat cycle 0002's v001 (bit-packed second-hit filter, baseline item size) as the base. Drop further admission-signal, recency-bit and hit-path-shortcut variants, because hit-rate is saturated and each costs latency. Aim the next cycle at latency work on the lookup, locking and bucket-scan paths, or at evaluating on a different trace. Repeat the latency measurement before trusting any result.
